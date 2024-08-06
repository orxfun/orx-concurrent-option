use crate::{concurrent_option::ConcurrentOption, IntoOption};
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};

impl<T> ConcurrentOption<T> {
    // &self

    pub fn is_some(&self, order: Ordering) -> bool {
        self.written.load(order)
    }

    pub fn is_none(&self, order: Ordering) -> bool {
        !self.written.load(order)
    }

    pub fn as_ref(&self, order: Ordering) -> Option<&T> {
        match self.written.load(order) {
            true => Some(unsafe { self.value.assume_init_ref() }),
            false => None,
        }
    }

    pub fn as_deref(&self, order: Ordering) -> Option<&<T as Deref>::Target>
    where
        T: Deref,
    {
        match self.written.load(order) {
            true => Some(unsafe { self.value.assume_init_ref() }),
            false => None,
        }
    }

    pub fn iter<'a>(&'a self, order: Ordering) -> crate::iter::Iter<'a, T> {
        let maybe = self.as_ref(order);
        crate::iter::Iter { maybe }
    }

    // &mut self

    pub fn as_deref_mut(&mut self) -> Option<&mut <T as Deref>::Target>
    where
        T: DerefMut,
    {
        self.as_mut().map(|x| x.deref_mut())
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self.written.load(Ordering::Relaxed) {
            true => Some(unsafe { self.value.assume_init_mut() }),
            false => None,
        }
    }

    pub fn take(&mut self) -> Option<T> {
        match self.is_some(Ordering::Relaxed) {
            false => None,
            true => {
                self.written.store(false, Ordering::Relaxed);
                Some(unsafe { self.value.assume_init_read() })
            }
        }
    }

    pub fn take_if<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnOnce(&mut T) -> bool,
    {
        match self.as_mut().map_or(false, predicate) {
            true => self.take(),
            false => None,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> crate::iter::IterMut<'a, T> {
        let maybe = self.as_mut();
        crate::iter::IterMut { maybe }
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        match self.written.load(Ordering::Relaxed) {
            true => {
                let x = unsafe { self.value.assume_init_mut() };
                let old = std::mem::replace(x, value);
                Some(old)
            }
            false => {
                self.value = MaybeUninit::new(value);
                self.written.store(true, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn insert(&mut self, value: T) -> &mut T {
        match self.written.load(Ordering::Relaxed) {
            true => {
                let x = unsafe { self.value.assume_init_mut() };
                let _ = std::mem::replace(x, value);
            }
            false => {
                self.value = MaybeUninit::new(value);
                self.written.store(true, Ordering::Relaxed);
            }
        }

        self.as_mut().expect("is guaranteed to be some")
    }

    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if !self.written.load(Ordering::Relaxed) {
            self.value = MaybeUninit::new(f());
            self.written.store(true, Ordering::Relaxed);
        }

        self.as_mut().expect("is guaranteed to be some")
    }

    // self

    pub fn expect(mut self, msg: &str) -> T {
        self.take().expect(msg)
    }

    pub fn unwrap(self) -> T {
        self.expect("called `unwrap()` on a `None` value")
    }

    pub fn unwrap_or(mut self, default: T) -> T {
        self.take().unwrap_or(default)
    }

    pub fn unwrap_or_default(mut self) -> T
    where
        T: Default,
    {
        self.take().unwrap_or_default()
    }

    pub fn unwrap_or_else<F>(mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.take().unwrap_or_else(f)
    }

    pub unsafe fn unwrap_unchecked(self) -> T {
        self.value.assume_init_read()
    }

    pub fn and<U>(mut self, other: impl IntoOption<U>) -> Option<U> {
        self.take().and(other.into_option())
    }

    pub fn and_then<U, F>(mut self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>,
    {
        self.take().and_then(f)
    }

    pub fn filter<P>(mut self, predicate: P) -> Option<T>
    where
        P: FnOnce(&T) -> bool,
    {
        self.take().and_then(|x| match predicate(&x) {
            true => Some(x),
            false => None,
        })
    }

    pub fn is_some_and(mut self, f: impl FnOnce(T) -> bool) -> bool {
        match self.take() {
            None => false,
            Some(x) => f(x),
        }
    }

    pub fn map<U, F>(mut self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        self.take().map(f)
    }

    pub fn map_or<U, F>(mut self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        self.take().map_or(default, f)
    }

    pub fn map_or_else<U, D, F>(mut self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        self.take().map_or_else(default, f)
    }

    pub fn ok_or<E>(mut self, err: E) -> Result<T, E> {
        self.take().ok_or(err)
    }

    pub fn ok_or_else<E, F>(mut self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        self.take().ok_or_else(err)
    }

    pub fn or(mut self, other: impl IntoOption<T>) -> Option<T> {
        self.take().or(other.into_option())
    }

    pub fn or_else<F, O>(mut self, f: F) -> Option<T>
    where
        O: IntoOption<T>,
        F: FnOnce() -> O,
    {
        self.take().or_else(|| f().into_option())
    }

    pub fn xor(mut self, other: impl IntoOption<T>) -> Option<T> {
        self.take().xor(other.into_option())
    }

    pub fn zip<U>(mut self, other: impl IntoOption<U>) -> Option<(T, U)> {
        match (self.take(), other.into_option().take()) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }
}

impl<T> ConcurrentOption<&T> {
    pub fn cloned(mut self) -> Option<T>
    where
        T: Clone,
    {
        self.take().cloned()
    }

    pub fn copied(mut self) -> Option<T>
    where
        T: Copy,
    {
        self.take().copied()
    }
}

impl<T> ConcurrentOption<ConcurrentOption<T>> {
    pub fn flatten(mut self) -> Option<T> {
        self.take().and_then(|mut x| x.take())
    }
}

impl<T> ConcurrentOption<Option<T>> {
    pub fn flatten(mut self) -> Option<T> {
        self.take().and_then(|x| x)
    }
}

impl<T, U> ConcurrentOption<(T, U)> {
    pub fn unzip(mut self) -> (Option<T>, Option<U>) {
        match self.take() {
            Some((x, y)) => (Some(x), Some(y)),
            None => (None, None),
        }
    }
}
