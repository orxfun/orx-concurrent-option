use crate::{concurrent_option::ConcurrentOption, states::*, IntoOption};
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};

impl<T> ConcurrentOption<T> {
    // &self

    /// Returns `true` if the option is a Some variant.
    ///
    /// See [`is_some_with_order`] to explicitly set the ordering.
    ///
    /// [`is_some_with_order`]: ConcurrentOption::is_some_with_order
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::some(2);
    /// assert_eq!(x.is_some(), true);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.is_some(), false);
    /// ```
    #[inline]
    pub fn is_some(&self) -> bool {
        self.is_some_with_order(ORDER_LOAD)
    }

    /// Returns `true` if the option is a None variant.
    ///
    /// See [`is_none_with_order`] to explicitly set the ordering.
    ///
    /// [`is_none_with_order`]: ConcurrentOption::is_none_with_order
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
        self.is_none_with_order(ORDER_LOAD)
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
        match self.mut_handle(SOME, SOME) {
            Some(_handle) => {
                let x = &*self.value.get();
                Some(x.assume_init_ref())
            }
            None => None,
        }
    }

    /// Converts from `Option<T>` (or `&Option<T>`) to `Option<&T::Target>`.
    ///
    /// Leaves the original Option in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    ///
    /// See [`as_deref_with_order`] to explicitly set the ordering.
    ///
    /// [`as_deref_with_order`]: ConcurrentOption::as_deref_with_order
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::some("hey".to_owned());
    /// assert_eq!(x.as_deref(), Some("hey"));
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::none();
    /// assert_eq!(x.as_deref(), None);
    /// ```
    #[inline]
    pub fn as_deref(&self) -> Option<&<T as Deref>::Target>
    where
        T: Deref,
    {
        self.as_deref_with_order(ORDER_LOAD)
    }

    /// Returns an iterator over the possibly contained value; yields
    /// * the single element if the option is of Some variant;
    /// * no elements otherwise.
    ///
    /// See [`iter_with_order`] to explicitly set the ordering.
    ///
    /// [`iter_with_order`]: ConcurrentOption::iter_with_order
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
    ///     assert_eq!(iter.len(), 0);
    ///     assert!(iter.next().is_none());
    ///     assert!(iter.next().is_none());
    /// }
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// validate(x.iter());
    /// validate(x.iter().rev());
    /// validate((&x).into_iter());
    /// ```
    #[inline]
    pub fn iter(&self) -> crate::iter::Iter<'_, T> {
        self.iter_with_order(ORDER_LOAD)
    }

    // &self - with-order

    /// Returns `true` if the option is a Some variant.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::some(2);
    /// assert_eq!(x.is_some_with_order(Ordering::Relaxed), true);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.is_some_with_order(Ordering::SeqCst), false);
    /// ```
    pub fn is_some_with_order(&self, order: Ordering) -> bool {
        self.state.load(order) == SOME
    }

    /// Returns `true` if the option is a None variant.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::some(2);
    /// assert_eq!(x.is_none_with_order(Ordering::Relaxed), false);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.is_none_with_order(Ordering::SeqCst), true);
    /// ```
    pub fn is_none_with_order(&self, order: Ordering) -> bool {
        self.state.load(order) == NONE
    }

    /// Converts from `&Option<T>` to `Option<&T>`.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// assert_eq!(unsafe { x.as_ref_with_order(Ordering::Relaxed) }, Some(&3.to_string()));
    ///
    /// _ = x.take();
    /// assert_eq!(unsafe { x.as_ref_with_order(Ordering::Acquire) }, None);
    /// ```
    pub unsafe fn as_ref_with_order(&self, order: Ordering) -> Option<&T> {
        match self.state.load(order) {
            SOME => {
                let x = &*self.value.get();
                Some(x.assume_init_ref())
            }
            _ => None,
        }
    }

    /// Converts from `Option<T>` (or `&Option<T>`) to `Option<&T::Target>`.
    ///
    /// Leaves the original Option in-place, creating a new one with a reference
    /// to the original one, additionally coercing the contents via [`Deref`].
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::some("hey".to_owned());
    /// assert_eq!(x.as_deref_with_order(Ordering::Acquire), Some("hey"));
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::none();
    /// assert_eq!(x.as_deref_with_order(Ordering::SeqCst), None);
    /// ```
    pub fn as_deref_with_order(&self, order: Ordering) -> Option<&<T as Deref>::Target>
    where
        T: Deref,
    {
        match self.state.load(order) {
            SOME => Some(unsafe { self.value_ref() }),
            _ => None,
        }
    }

    /// Returns an iterator over the possibly contained value; yields
    /// * the single element if the option is of Some variant;
    /// * no elements otherwise.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
    ///     assert_eq!(iter.len(), 0);
    ///     assert!(iter.next().is_none());
    ///     assert!(iter.next().is_none());
    /// }
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// validate(x.iter_with_order(Ordering::SeqCst));
    /// validate(x.iter_with_order(Ordering::Relaxed).rev());
    /// validate((&x).into_iter());
    /// ```
    pub fn iter_with_order(&self, order: Ordering) -> crate::iter::Iter<'_, T> {
        let maybe = unsafe { self.as_ref_with_order(order) };
        crate::iter::Iter { maybe }
    }

    // &mut self

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
        match self.is_some_with_order(Ordering::Relaxed) {
            false => None,
            true => {
                self.state.store(NONE, Ordering::Relaxed);
                let x = unsafe { &mut *self.value.get() };
                Some(unsafe { x.assume_init_read() })
            }
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

    // self

    /// Returns the contained Some value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is a None with a custom panic message provided by
    /// `message`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("value");
    /// assert_eq!(x.expect("fruits are healthy"), "value");
    /// ```
    ///
    /// ```should_panic
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// x.expect("fruits are healthy"); // panics with `fruits are healthy`
    /// ```
    pub fn expect(mut self, message: &str) -> T {
        self.exclusive_take().expect(message)
    }

    /// Returns the contained Some value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the None
    /// case explicitly, or call [`ConcurrentOption::unwrap_or`], [`ConcurrentOption::unwrap_or_else`], or
    /// [`ConcurrentOption::unwrap_or_default`].
    ///
    /// # Panics
    ///
    /// Panics if the self value equals None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```should_panic
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.unwrap(), "air"); // fails
    /// ```
    pub fn unwrap(self) -> T {
        self.expect("called `unwrap()` on a `None` value")
    }

    /// Returns the contained Some value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`ConcurrentOption::unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// assert_eq!(ConcurrentOption::some("car").unwrap_or("bike"), "car");
    /// assert_eq!(ConcurrentOption::none().unwrap_or("bike"), "bike");
    /// ```
    pub fn unwrap_or(mut self, default: T) -> T {
        self.exclusive_take().unwrap_or(default)
    }

    /// Returns the contained Some value or a default.
    ///
    /// Consumes the `self` argument then, if Some, returns the contained
    /// value, otherwise if None, returns the [default value] for that
    /// type.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y: ConcurrentOption<u32> = ConcurrentOption::some(12);
    ///
    /// assert_eq!(x.unwrap_or_default(), 0);
    /// assert_eq!(y.unwrap_or_default(), 12);
    /// ```
    pub fn unwrap_or_default(mut self) -> T
    where
        T: Default,
    {
        self.exclusive_take().unwrap_or_default()
    }

    /// Returns the contained Some value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let k = 10;
    /// assert_eq!(ConcurrentOption::some(4).unwrap_or_else(|| 2 * k), 4);
    /// assert_eq!(ConcurrentOption::none().unwrap_or_else(|| 2 * k), 20);
    /// ```
    pub fn unwrap_or_else<F>(mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.exclusive_take().unwrap_or_else(f)
    }

    /// Returns the contained Some value, consuming the `self` value,
    /// without checking that the value is not None.
    ///
    /// # Safety
    ///
    /// Calling this method on None is *[undefined behavior]*.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("air");
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, "air");
    /// ```
    ///
    /// ```no_run
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, "air"); // Undefined behavior!
    /// ```
    pub unsafe fn unwrap_unchecked(self) -> T {
        self.state.store(NONE, Ordering::Relaxed);
        let x = &mut *self.value.get();
        x.assume_init_read()
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
    pub fn and<U>(mut self, other: impl IntoOption<U>) -> Option<U> {
        self.exclusive_take().and(other.into_option())
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
    /// fn sq_then_to_string(x: u32) -> Option<String> {
    ///     x.checked_mul(x).map(|sq| sq.to_string())
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
    /// fn sq_then_to_string(x: u32) -> ConcurrentOption<String> {
    ///     x.checked_mul(x).map(|sq| sq.to_string()).into()
    /// }
    ///
    /// assert_eq!(ConcurrentOption::some(2).and_then(sq_then_to_string), Some(4.to_string()));
    /// assert_eq!(ConcurrentOption::some(1_000_000).and_then(sq_then_to_string), None); // overflowed!
    /// assert_eq!(ConcurrentOption::none().and_then(sq_then_to_string), None);
    /// ```
    pub fn and_then<U, V, F>(mut self, f: F) -> Option<U>
    where
        V: IntoOption<U>,
        F: FnOnce(T) -> V,
    {
        self.exclusive_take().and_then(|x| f(x).into_option())
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
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// fn is_even(n: &i32) -> bool {
    ///     n % 2 == 0
    /// }
    ///
    /// assert_eq!(ConcurrentOption::none().filter(is_even), None);
    /// assert_eq!(ConcurrentOption::some(3).filter(is_even), None);
    /// assert_eq!(ConcurrentOption::some(4).filter(is_even), Some(4));
    /// ```
    pub fn filter<P>(mut self, predicate: P) -> Option<T>
    where
        P: FnOnce(&T) -> bool,
    {
        self.exclusive_take().and_then(|x| match predicate(&x) {
            true => Some(x),
            false => None,
        })
    }

    /// Returns `true` if the option is a Some and the value inside of it matches a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(2);
    /// assert_eq!(x.is_some_and(|x| x > 1), true);
    ///
    /// let x = ConcurrentOption::some(0);
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    ///
    /// let x: ConcurrentOption<i32> = ConcurrentOption::none();
    /// assert_eq!(x.is_some_and(|x| x > 1), false);
    /// ```
    pub fn is_some_and(mut self, f: impl FnOnce(T) -> bool) -> bool {
        match self.exclusive_take() {
            None => false,
            Some(x) => f(x),
        }
    }

    /// Maps an `ConcurrentOption<T>` to `Option<U>` by applying a function to a contained value (if `Some`) or returns `None` (if `None`).
    ///
    /// # Examples
    ///
    /// Calculates the length of an <code>Option<[String]></code> as an
    /// <code>Option<[usize]></code>, consuming the original:
    ///
    /// [String]: ../../std/string/struct.String.html "String"
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let maybe_some_string = ConcurrentOption::some(String::from("Hello, World!"));
    /// // `Option::map` takes self *by value*, consuming `maybe_some_string`
    /// let maybe_some_len = maybe_some_string.map(|s| s.len());
    /// assert_eq!(maybe_some_len, Some(13));
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.map(|s| s.len()), None);
    /// ```
    pub fn map<U, F>(mut self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        self.exclusive_take().map(f)
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
    pub fn map_or<U, F>(mut self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        self.exclusive_take().map_or(default, f)
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
    pub fn map_or_else<U, D, F>(mut self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        self.exclusive_take().map_or_else(default, f)
    }

    /// Transforms the `ConcurrentOption<T>` into a [`Result<T, E>`], mapping Some(v) to
    /// [`Ok(v)`] and None to [`Err(err)`].
    ///
    /// Arguments passed to `ok_or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`ok_or_else`], which is
    /// lazily evaluated.
    ///
    /// [`Ok(v)`]: Ok
    /// [`Err(err)`]: Err
    /// [`ok_or_else`]: ConcurrentOption::ok_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("foo");
    /// assert_eq!(x.ok_or(0), Ok("foo"));
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.ok_or(0), Err(0));
    /// ```
    pub fn ok_or<E>(mut self, err: E) -> Result<T, E> {
        self.exclusive_take().ok_or(err)
    }

    /// Transforms the `Option<T>` into a [`Result<T, E>`], mapping `Some(v)` to
    /// [`Ok(v)`] and `None` to [`Err(err())`].
    ///
    /// [`Ok(v)`]: Ok
    /// [`Err(err())`]: Err
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("foo");
    /// assert_eq!(x.ok_or_else(|| 0), Ok("foo"));
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.ok_or_else(|| 0), Err(0));
    /// ```
    pub fn ok_or_else<E, F>(mut self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        self.exclusive_take().ok_or_else(err)
    }

    /// Returns the option if it contains a value, otherwise returns `other`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`or_else`], which is
    /// lazily evaluated.
    ///
    /// [`or_else`]: ConcurrentOption::or_else
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(2);
    /// let y = ConcurrentOption::none();
    /// assert_eq!(x.or(y), Some(2));
    ///
    /// let x = ConcurrentOption::none();
    /// let y = Some(100);
    /// assert_eq!(x.or(y), Some(100));
    ///
    /// let x = ConcurrentOption::some(2);
    /// let y = Some(100);
    /// assert_eq!(x.or(y), Some(2));
    ///
    /// let x: ConcurrentOption<i32> = ConcurrentOption::none();
    /// let y = None;
    /// assert_eq!(x.or(y), None);
    /// ```
    pub fn or(mut self, other: impl IntoOption<T>) -> Option<T> {
        self.exclusive_take().or(other.into_option())
    }

    /// Returns the option if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// fn nobody() -> ConcurrentOption<&'static str> { ConcurrentOption::none() }
    /// fn vikings() -> ConcurrentOption<&'static str> { ConcurrentOption::some("vikings") }
    ///
    /// assert_eq!(ConcurrentOption::some("barbarians").or_else(vikings), Some("barbarians"));
    /// assert_eq!(ConcurrentOption::none().or_else(vikings), Some("vikings"));
    /// assert_eq!(ConcurrentOption::none().or_else(nobody), None);
    /// ```
    pub fn or_else<F, O>(mut self, f: F) -> Option<T>
    where
        O: IntoOption<T>,
        F: FnOnce() -> O,
    {
        self.exclusive_take().or_else(|| f().into_option())
    }

    /// Returns `Some` if exactly one of `self`, `other` is `Some`, otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(2);
    /// let y: Option<u32> = None;
    /// assert_eq!(x.xor(y), Some(2));
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y = Some(2);
    /// assert_eq!(x.xor(y), Some(2));
    ///
    /// let x = ConcurrentOption::some(2);
    /// let y = Some(2);
    /// assert_eq!(x.xor(y), None);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.xor(y), None);
    /// ```
    pub fn xor(mut self, other: impl IntoOption<T>) -> Option<T> {
        self.exclusive_take().xor(other.into_option())
    }

    /// Zips `self` with another option (`Option` or `ConcurrentOption`).
    ///
    /// If `self` is `Some(s)` and `other` is `Some(o)`, this method returns `Some((s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(1);
    /// let y = Some("hi");
    /// let z = ConcurrentOption::<u8>::none();
    ///
    /// assert_eq!(x.clone().zip(y), Some((1, "hi")));
    /// assert_eq!(x.zip(z), None);
    /// ```
    pub fn zip<U>(mut self, other: impl IntoOption<U>) -> Option<(T, U)> {
        match (self.exclusive_take(), other.into_option().take()) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
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
    /// use std::sync::atomic::Ordering;
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

impl<T, U> ConcurrentOption<(T, U)> {
    /// Unzips a ConcurrentOption containing a tuple of two Option's.
    ///
    /// If `self` is `ConcurrentOption::some((a, b))` this method returns `(Some(a), Some(b))`.
    /// Otherwise, `(None, None)` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some((1, "hi"));
    /// let y = ConcurrentOption::<(u8, u32)>::none();
    ///
    /// assert_eq!(x.unzip(), (Some(1), Some("hi")));
    /// assert_eq!(y.unzip(), (None, None));
    /// ```
    pub fn unzip(mut self) -> (Option<T>, Option<U>) {
        match self.exclusive_take() {
            Some((x, y)) => (Some(x), Some(y)),
            None => (None, None),
        }
    }
}
