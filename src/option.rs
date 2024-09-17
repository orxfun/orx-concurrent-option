use crate::{concurrent_option::ConcurrentOption, states::*, IntoOption};
use core::{mem::MaybeUninit, ops::Deref};

impl<T> ConcurrentOption<T> {
    // &self

    /// Returns `true` if the option is a Some variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::some(2);
    /// assert_eq!(x.is_some(), true);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.is_some(), false);
    /// ```
    #[inline]
    pub fn is_some(&self) -> bool {
        self.state.load(ORDER_LOAD) == SOME
    }

    /// Returns `true` if the option is a None variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::some(2);
    /// assert_eq!(x.is_none(), false);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.is_none(), true);
    /// ```
    #[inline]
    pub fn is_none(&self) -> bool {
        self.state.load(ORDER_LOAD) != SOME
    }

    /// Partially thread safe method to convert from `&Option<T>` to `Option<&T>`.
    ///
    /// # Safety
    ///
    /// Note that creating a valid reference part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent writes while holding onto this reference.
    ///   * One such case is using `as_ref` together with `initialize_when_none` method.
    /// This is perfectly safe since the value will be written only once,
    /// and `as_ref` returns a valid reference only after the value is initialized.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));
    ///
    /// _ = x.take();
    /// assert_eq!(unsafe { x.as_ref() }, None);
    /// ```
    pub unsafe fn as_ref(&self) -> Option<&T> {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = &*self.value.get();
                Some(x.assume_init_ref())
            }
            None => None,
        }
    }

    /// Partially thread safe method to convert from `Option<T>` (or `&Option<T>`) to `Option<&T::Target>`.
    ///
    /// Leaves the original Option in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    ///
    /// # Safety
    ///
    /// Note that creating a valid reference part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent writes while holding onto this reference.
    ///   * One such case is using `as_ref` together with `initialize_when_none` method.
    /// This is perfectly safe since the value will be written only once,
    /// and `as_ref` returns a valid reference only after the value is initialized.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::some("hey".to_owned());
    /// assert_eq!(unsafe { x.as_deref() }, Some("hey"));
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::none();
    /// assert_eq!(unsafe { x.as_deref() }, None);
    /// ```
    pub unsafe fn as_deref(&self) -> Option<&<T as Deref>::Target>
    where
        T: Deref,
    {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = &*self.value.get();
                Some(x.assume_init_ref())
            }
            None => None,
        }
    }

    /// Partially thread safe method to return an iterator over the possibly contained value; yields
    /// * the single element if the option is of Some variant;
    /// * no elements otherwise.
    ///
    /// # Safety
    ///
    /// Note that creating a valid reference part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent writes while holding onto this reference.
    ///   * One such case is using `as_ref` together with `initialize_when_none` method.
    /// This is perfectly safe since the value will be written only once,
    /// and `as_ref` returns a valid reference only after the value is initialized.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
    ///     assert_eq!(iter.len(), 0);
    ///     assert!(iter.next().is_none());
    ///     assert!(iter.next().is_none());
    /// }
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// validate(unsafe { x.iter() });
    /// validate(unsafe { x.iter() }.rev());
    /// validate((&x).into_iter());
    /// ```
    pub unsafe fn iter(&self) -> crate::iter::Iter<'_, T> {
        crate::iter::Iter {
            maybe: self.as_ref(),
        }
    }

    /// Thread safe method to map the reference of the underlying value with the given function `f`.
    ///
    /// Returns
    /// * None if the option is None
    /// * `f(&value)` if the option is Some(value)
    ///
    /// # Concurrency Notes
    ///
    /// Notice that `map` is a composition of `as_ref` and `map`.
    /// However, it is stronger in terms of thread safety since the access to the value is controlled
    /// and a reference to the underlying value is not leaked outside the option.
    ///
    /// Therefore, `map` must be preferred in a concurrent program:
    /// * the map operation via `map` guarantees that the underlying value will not be updated before the operation; while
    /// * the alternative approach with `as_ref` is subject to data race if the state of the optional is concurrently being
    /// updated by methods such as `take`.
    ///   * an exception to this is the `initialize_if_none` method which fits very well the initialize-once scenarios;
    /// here, `as_ref` and `initialize_if_none` can safely be called concurrently from multiple threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let len = x.map(|x| x.len());
    /// assert_eq!(len, None);
    ///
    /// let x = ConcurrentOption::some("foo".to_string());
    /// let len = x.map(|x| x.len());
    /// assert_eq!(len, Some(3));
    /// ```
    pub fn map<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> U,
    {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                let x = unsafe { MaybeUninit::assume_init_ref(x) };
                Some(f(x))
            }
            None => None,
        }
    }

    /// Returns the provided default result (if none),
    /// or applies a function to the contained value (if any).
    ///
    /// Arguments passed to `map_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`map_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`map_or_else`]: ConcurrentOption::map_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("foo");
    /// assert_eq!(x.map_or(42, |v| v.len()), 3);
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.map_or(42, |v| v.len()), 42);
    /// ```
    pub fn map_or<U, F>(&self, default: U, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        self.map(f).unwrap_or(default)
    }

    /// Computes a default function result (if none), or
    /// applies a different function to the contained value (if any).
    ///
    /// # Basic examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let k = 21;
    ///
    /// let x = ConcurrentOption::some("foo");
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 3);
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 42);
    /// ```
    pub fn map_or_else<U, D, F>(&self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&T) -> U,
    {
        self.map(f).unwrap_or_else(default)
    }

    /// Thread safe method that returns `true` if the option is a Some and the value inside of it matches a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(2);
    /// assert_eq!(x.is_some_and(|x| *x > 1), true);
    ///
    /// let x = ConcurrentOption::some(0);
    /// assert_eq!(x.is_some_and(|x| *x > 1), false);
    ///
    /// let x: ConcurrentOption<i32> = ConcurrentOption::none();
    /// assert_eq!(x.is_some_and(|x| *x > 1), false);
    /// ```
    #[inline]
    pub fn is_some_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        self.map(|x| f(x)).unwrap_or(false)
    }

    /// Returns None if the option is None, otherwise returns `other`.
    ///
    /// Arguments passed to `and` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`and_then`], which is
    /// lazily evaluated.
    ///
    /// [`and_then`]: ConcurrentOption::and_then
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(2);
    /// let y: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.and(y), None);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y = ConcurrentOption::some("foo");
    /// assert_eq!(x.and(y), None);
    ///
    /// let x = ConcurrentOption::some(2);
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), Some("foo"));
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    /// ```
    pub fn and<U>(&self, other: impl IntoOption<U>) -> Option<U> {
        self.map(|_| ()).and(other.into_option())
    }

    /// Returns None if the option is None, otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// Some languages call this operation flatmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// fn sq_then_to_string(x: &u32) -> Option<String> {
    ///     x.checked_mul(*x).map(|sq| sq.to_string())
    /// }
    ///
    /// assert_eq!(ConcurrentOption::some(2).and_then(sq_then_to_string), Some(4.to_string()));
    /// assert_eq!(ConcurrentOption::some(1_000_000).and_then(sq_then_to_string), None); // overflowed!
    /// assert_eq!(ConcurrentOption::none().and_then(sq_then_to_string), None);
    /// ```
    ///
    /// Since `ConcurrentOption` also implements `IntoOption`; and_then can also be called with
    /// a function returning a concurrent option.
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// fn sq_then_to_string(x: &u32) -> ConcurrentOption<String> {
    ///     x.checked_mul(*x).map(|sq| sq.to_string()).into()
    /// }
    ///
    /// assert_eq!(ConcurrentOption::some(2).and_then(sq_then_to_string), Some(4.to_string()));
    /// assert_eq!(ConcurrentOption::some(1_000_000).and_then(sq_then_to_string), None); // overflowed!
    /// assert_eq!(ConcurrentOption::none().and_then(sq_then_to_string), None);
    /// ```
    pub fn and_then<U, V, F>(&self, f: F) -> Option<U>
    where
        V: IntoOption<U>,
        F: FnOnce(&T) -> V,
    {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                let x = unsafe { MaybeUninit::assume_init_ref(x) };
                f(x).into_option()
            }
            None => None,
        }
    }

    /// Returns None if the option is None, otherwise calls `predicate`
    /// with the wrapped value and returns:
    ///
    /// - Some(t) if `predicate` returns `true` (where `t` is the wrapped
    ///   value), and
    /// - None if `predicate` returns `false`.
    ///
    /// This function works similar to [`Iterator::filter()`]. You can imagine
    /// the `Option<T>` being an iterator over one or zero elements. `filter()`
    /// lets you decide which elements to keep.
    ///
    /// # Safety
    ///
    /// Note that creating a valid reference part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent writes while holding onto this reference.
    ///   * One such case is using `as_ref` together with `initialize_when_none` method.
    /// This is perfectly safe since the value will be written only once,
    /// and `as_ref` returns a valid reference only after the value is initialized.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// fn is_even(n: &i32) -> bool {
    ///     n % 2 == 0
    /// }
    /// unsafe
    /// {
    ///     assert_eq!(ConcurrentOption::none().filter(is_even), None);
    ///     assert_eq!(ConcurrentOption::some(3).filter(is_even), None);
    ///     assert_eq!(ConcurrentOption::some(4).filter(is_even), Some(&4));
    /// }
    /// ```
    pub unsafe fn filter<P>(&self, predicate: P) -> Option<&T>
    where
        P: FnOnce(&T) -> bool,
    {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                let x = unsafe { MaybeUninit::assume_init_ref(x) };
                match predicate(x) {
                    true => Some(x),
                    false => None,
                }
            }
            None => None,
        }
    }
}

impl<T> ConcurrentOption<&T> {
    /// Maps an `ConcurrentOption<&T>` to an `Option<T>` by cloning the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
    ///
    /// let x = 12;
    /// let opt_x = ConcurrentOption::some(&x);
    /// assert_eq!(unsafe { opt_x.as_ref() }, Some(&&12));
    ///
    /// let cloned = opt_x.cloned();
    /// assert_eq!(cloned, Some(12));
    /// ```
    pub fn cloned(mut self) -> Option<T>
    where
        T: Clone,
    {
        self.exclusive_take().cloned()
    }

    /// Maps an `ConcurrentOption<&T>` to an `Option<T>` by copying the contents of the
    /// option.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = 12;
    /// let opt_x = ConcurrentOption::some(&x);
    /// assert_eq!(unsafe { opt_x.as_ref() }, Some(&&12));
    ///
    /// let copied = opt_x.copied();
    /// assert_eq!(copied, Some(12));
    /// ```
    pub fn copied(mut self) -> Option<T>
    where
        T: Copy,
    {
        self.exclusive_take().copied()
    }
}

impl<T> ConcurrentOption<ConcurrentOption<T>> {
    /// Converts from `ConcurrentOption<ConcurrentOption<T>>` to `Option<T>`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<ConcurrentOption<u32>> = ConcurrentOption::some(ConcurrentOption::some(6));
    /// assert_eq!(Some(6), x.flatten());
    ///
    /// let x: ConcurrentOption<ConcurrentOption<u32>> = ConcurrentOption::some(ConcurrentOption::none());
    /// assert_eq!(None, x.flatten());
    ///
    /// let x: ConcurrentOption<ConcurrentOption<u32>> = ConcurrentOption::none();
    /// assert_eq!(None, x.flatten());
    /// ```
    pub fn flatten(mut self) -> Option<T> {
        self.exclusive_take().and_then(|mut x| x.exclusive_take())
    }
}

impl<T> ConcurrentOption<Option<T>> {
    /// Converts from `ConcurrentOption<Option<T>>` to `Option<T>`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<Option<u32>> = ConcurrentOption::some(Some(6));
    /// assert_eq!(Some(6), x.flatten());
    ///
    /// let x: ConcurrentOption<Option<u32>> = ConcurrentOption::some(None);
    /// assert_eq!(None, x.flatten());
    ///
    /// let x: ConcurrentOption<Option<u32>> = ConcurrentOption::none();
    /// assert_eq!(None, x.flatten());
    /// ```
    pub fn flatten(mut self) -> Option<T> {
        self.exclusive_take().and_then(|x| x)
    }
}
