use crate::{states::*, ConcurrentOption};
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};

impl<T> ConcurrentOption<T> {
    /// Converts from `Option<T>` (or `&mut Option<T>`) to `Option<&mut T::Target>`.
    ///
    /// Leaves the original `Option` in-place, creating a new one containing a mutable reference to
    /// the inner type's [`Deref::Target`] type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x: ConcurrentOption<String> = ConcurrentOption::some("hey".to_owned());
    /// assert_eq!(x.exclusive_as_deref_mut().map(|x| {
    ///     x.make_ascii_uppercase();
    ///     x
    /// }), Some("HEY".to_owned().as_mut_str()));
    /// ```
    pub fn exclusive_as_deref_mut(&mut self) -> Option<&mut <T as Deref>::Target>
    where
        T: DerefMut,
    {
        self.exclusive_as_mut().map(|x| x.deref_mut())
    }

    /// Converts from `&mut Option<T>` to `Option<&mut T>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::some(2);
    /// match x.exclusive_as_mut() {
    ///     Some(v) => *v = 42,
    ///     None => {},
    /// }
    /// assert_eq!(unsafe { x.as_ref() }, Some(&42));
    /// ```
    pub fn exclusive_as_mut(&mut self) -> Option<&mut T> {
        match self.state.load(Ordering::Relaxed) {
            SOME => Some(unsafe { (*self.value.get()).assume_init_mut() }),
            _ => None,
        }
    }

    /// Takes the value out of the option, leaving a None in its place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::some(42);
    /// let y = x.exclusive_take();
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert_eq!(y, Some(42));
    ///
    /// let mut x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y = x.exclusive_take();
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert_eq!(y, None);
    /// ```
    pub fn exclusive_take(&mut self) -> Option<T> {
        match self.state(Ordering::Relaxed) {
            State::Some => {
                self.state.store(NONE, Ordering::Relaxed);
                let x = unsafe { &mut *self.value.get() };
                Some(unsafe { x.assume_init_read() })
            }
            _ => None,
        }
    }

    /// Takes the value out of the option, but only if the predicate evaluates to
    /// `true` on a mutable reference to the value.
    ///
    /// In other words, replaces `self` with None if the predicate returns `true`.
    /// This method operates similar to [`ConcurrentOption::exclusive_take`] but conditional.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::some(42);
    ///
    /// let prev = x.exclusive_take_if(|v| if *v == 42 {
    ///     *v += 1;
    ///     false
    /// } else {
    ///     false
    /// });
    /// assert_eq!(x, ConcurrentOption::some(43));
    /// assert_eq!(prev, None);
    ///
    /// let prev = x.exclusive_take_if(|v| *v == 43);
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert_eq!(prev, Some(43));
    /// ```
    pub fn exclusive_take_if<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnOnce(&mut T) -> bool,
    {
        match self.exclusive_as_mut().map_or(false, predicate) {
            true => self.exclusive_take(),
            false => None,
        }
    }

    /// Returns a mutable iterator over the possibly contained value; yields
    /// * the single element if the option is of Some variant;
    /// * no elements otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::some(4);
    /// match x.exclusive_iter_mut().next() {
    ///     Some(v) => *v = 42,
    ///     None => {},
    /// }
    /// assert_eq!(x, ConcurrentOption::some(42));
    ///
    /// let mut x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.exclusive_iter_mut().next(), None);
    /// ```
    pub fn exclusive_iter_mut(&mut self) -> crate::iter::IterMut<'_, T> {
        let maybe = self.exclusive_as_mut();
        crate::iter::IterMut { maybe }
    }

    /// Replaces the actual value in the option by the value given in parameter,
    /// returning the old value if present,
    /// leaving a Some in its place without de-initializing either one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::some(2);
    /// let old = x.exclusive_replace(5);
    /// assert_eq!(x, ConcurrentOption::some(5));
    /// assert_eq!(old, Some(2));
    ///
    /// let mut x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let old = x.exclusive_replace(3);
    /// assert_eq!(x, ConcurrentOption::some(3));
    /// assert_eq!(old, None);
    /// ```
    pub fn exclusive_replace(&mut self, value: T) -> Option<T> {
        match self.state.load(Ordering::Relaxed) {
            SOME => {
                self.state.store(RESERVED, Ordering::Relaxed);
                let x = unsafe { (*self.value.get()).assume_init_mut() };
                let old = std::mem::replace(x, value);
                self.state.store(SOME, Ordering::Relaxed);
                Some(old)
            }
            NONE => {
                self.state.store(RESERVED, Ordering::Relaxed);
                self.value = MaybeUninit::new(value).into();
                self.state.store(SOME, Ordering::Relaxed);
                None
            }
            _ => panic!("ConcurrentOption value is `replace`d while its value is being written."),
        }
    }

    /// Inserts `value` into the option, then returns a mutable reference to it.
    ///
    /// If the option already contains a value, the old value is dropped.
    ///
    /// See also [`Option::get_or_insert`], which doesn't update the value if
    /// the option already contains Some.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut opt: ConcurrentOption<_> = ConcurrentOption::none();
    ///
    /// let val = opt.exclusive_insert(1);
    /// assert_eq!(*val, 1);
    /// assert_eq!(unsafe { opt.as_ref() }, Some(&1));
    ///
    /// let val = opt.exclusive_insert(2);
    /// assert_eq!(*val, 2);
    /// *val = 3;
    /// assert_eq!(opt.unwrap(), 3);
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn exclusive_insert(&mut self, value: T) -> &mut T {
        match self.state.load(Ordering::Relaxed) {
            SOME => {
                self.state.store(RESERVED, Ordering::Relaxed);
                let x = unsafe { (*self.value.get()).assume_init_mut() };
                let _ = std::mem::replace(x, value);
                self.state.store(SOME, Ordering::Relaxed);
            }
            NONE => {
                self.state.store(RESERVED, Ordering::Relaxed);
                self.value = MaybeUninit::new(value).into();
                self.state.store(SOME, Ordering::Relaxed);
            }
            _ => panic!("ConcurrentOption value is `insert`ed while its value is being written."),
        }

        self.exclusive_as_mut().expect("should be some")
    }

    /// Inserts `value` into the option if it is None, then
    /// returns a mutable reference to the contained value.
    ///
    /// See also [`ConcurrentOption::insert`], which updates the value even if
    /// the option already contains Some.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::none();
    ///
    /// {
    ///     let y: &mut u32 = x.exclusive_get_or_insert(5);
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, ConcurrentOption::some(7));
    /// ```
    pub fn exclusive_get_or_insert(&mut self, value: T) -> &mut T {
        self.exclusive_get_or_insert_with(|| value)
    }

    /// Inserts a value computed from `f` into the option if it is None,
    /// then returns a mutable reference to the contained value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let mut x = ConcurrentOption::none();
    ///
    /// {
    ///     let y: &mut u32 = x.exclusive_get_or_insert_with(|| 5);
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, ConcurrentOption::some(7));
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn exclusive_get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        match self.state.load(Ordering::Relaxed) {
            SOME => self.exclusive_as_mut().expect("is guaranteed to be some"),
            NONE => {
                self.state.store(RESERVED, Ordering::Relaxed);
                self.value = MaybeUninit::new(f()).into();
                self.state.store(SOME, Ordering::Relaxed);
                self.exclusive_as_mut().expect("is guaranteed to be some")
            }
            _ => panic!(
                "ConcurrentOption `get_or_insert_with` is called while its value is being written."
            ),
        }
    }
}
