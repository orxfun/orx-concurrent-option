use crate::{states::*, ConcurrentOption};
use core::{ops::Deref, sync::atomic::Ordering};

impl<T> ConcurrentOption<T> {
    /// Loads and returns the concurrent state of the option with the given `order`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::some(2);
    /// assert_eq!(x.state(Ordering::Relaxed), State::Some);
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// assert_eq!(x.state(Ordering::SeqCst), State::None);
    /// ```
    pub fn state(&self, order: Ordering) -> State {
        State::new(self.state.load(order))
    }

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
    pub fn is_some_with_order(&self, order: Ordering) -> bool {
        self.state.load(order) == SOME
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
    pub fn is_none_with_order(&self, order: Ordering) -> bool {
        self.state.load(order) != SOME
    }

    /// Converts from `&Option<T>` to `Option<&T>`.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
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
    /// use core::sync::atomic::Ordering;
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
    /// use core::sync::atomic::Ordering;
    ///
    /// unsafe
    /// {
    ///     let x: ConcurrentOption<String> = ConcurrentOption::some("hey".to_owned());
    ///     assert_eq!(x.as_deref_with_order(Ordering::Acquire), Some("hey"));
    ///
    ///     let x: ConcurrentOption<String> = ConcurrentOption::none();
    ///     assert_eq!(x.as_deref_with_order(Ordering::SeqCst), None);
    /// }
    /// ```
    pub unsafe fn as_deref_with_order(&self, order: Ordering) -> Option<&<T as Deref>::Target>
    where
        T: Deref,
    {
        match self.state.load(order) {
            SOME => {
                let x = &*self.value.get();
                Some(x.assume_init_ref())
            }
            _ => None,
        }
    }

    /// Returns an iterator over the possibly contained value; yields
    /// * the single element if the option is of Some variant;
    /// * no elements otherwise.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
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
    /// use core::sync::atomic::Ordering;
    ///
    /// fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
    ///     assert_eq!(iter.len(), 0);
    ///     assert!(iter.next().is_none());
    ///     assert!(iter.next().is_none());
    /// }
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// unsafe
    /// {
    /// validate(x.iter_with_order(Ordering::SeqCst));
    ///     validate(x.iter_with_order(Ordering::Relaxed).rev());
    ///     validate((&x).into_iter());
    /// }
    /// ```
    pub unsafe fn iter_with_order(&self, order: Ordering) -> crate::iter::Iter<'_, T> {
        let maybe = unsafe { self.as_ref_with_order(order) };
        crate::iter::Iter { maybe }
    }

    /// Clones the concurrent option with the desired `order` into an Option.
    ///
    /// Note that the `Clone` trait implementation clones the concurrent option with the default ordering.
    ///
    /// You may use `clone_with_order` in order to clone with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
    ///
    /// let mut x = ConcurrentOption::some(42);
    /// let y = x.clone_with_order(Ordering::SeqCst);
    /// assert_eq!(x.take(), y);
    /// ```
    pub fn clone_with_order(&self, order: Ordering) -> Option<T>
    where
        T: Clone,
    {
        unsafe { self.as_ref_with_order(order) }.cloned()
    }

    /// Returns whether or not self is equal to the `other` with the desired `order`.
    ///
    /// Note that the `PartialEq` trait implementation checks equality with the default ordering.
    ///
    /// You may use `eq_with_order` in order to check equality with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let o = Ordering::SeqCst;
    ///
    /// assert!(x.eq_with_order(&x, o));
    /// assert!(!x.eq_with_order(&y, o));
    /// assert!(!x.eq_with_order(&z, o));
    ///
    /// assert!(!z.eq_with_order(&x, o));
    /// assert!(!z.eq_with_order(&y, o));
    /// assert!(z.eq_with_order(&z, o));
    /// ```
    pub fn eq_with_order(&self, other: &Self, order: Ordering) -> bool
    where
        T: PartialEq,
    {
        match (unsafe { self.as_ref_with_order(order) }, unsafe {
            other.as_ref_with_order(order)
        }) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y),
            _ => false,
        }
    }

    /// Returns an ordering between `self` and `other` with the desired `order`.
    ///
    /// Note that the `PartialOrd` trait implementation checks equality with the default ordering.
    ///
    /// You may use `partial_cmp_with_order` in order to check equality with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let ord = core::sync::atomic::Ordering::SeqCst;
    ///
    /// assert_eq!(x.partial_cmp_with_order(&x, ord), Some(Equal));
    /// assert_eq!(x.partial_cmp_with_order(&y, ord), Some(Less));
    /// assert_eq!(x.partial_cmp_with_order(&z, ord), Some(Greater));
    ///
    /// assert_eq!(y.partial_cmp_with_order(&x, ord), Some(Greater));
    /// assert_eq!(y.partial_cmp_with_order(&y, ord), Some(Equal));
    /// assert_eq!(y.partial_cmp_with_order(&z, ord), Some(Greater));
    ///
    /// assert_eq!(z.partial_cmp_with_order(&x, ord), Some(Less));
    /// assert_eq!(z.partial_cmp_with_order(&y, ord), Some(Less));
    /// assert_eq!(z.partial_cmp_with_order(&z, ord), Some(Equal));
    /// ```
    pub fn partial_cmp_with_order(
        &self,
        other: &Self,
        order: Ordering,
    ) -> Option<core::cmp::Ordering>
    where
        T: PartialOrd,
    {
        use core::cmp::Ordering::*;

        match (unsafe { self.as_ref_with_order(order) }, unsafe {
            other.as_ref_with_order(order)
        }) {
            (Some(l), Some(r)) => l.partial_cmp(r),
            (Some(_), None) => Some(Greater),
            (None, Some(_)) => Some(Less),
            (None, None) => Some(Equal),
        }
    }

    /// Returns an ordering between `self` and `other` with the desired `order`.
    ///
    /// Note that the `Ord` trait implementation checks equality with the default ordering.
    ///
    /// You may use `cmp_with_order` in order to check equality with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let ord = core::sync::atomic::Ordering::SeqCst;
    ///
    /// assert_eq!(x.cmp_with_order(&x, ord), Equal);
    /// assert_eq!(x.cmp_with_order(&y, ord), Less);
    /// assert_eq!(x.cmp_with_order(&z, ord), Greater);
    ///
    /// assert_eq!(y.cmp_with_order(&x, ord), Greater);
    /// assert_eq!(y.cmp_with_order(&y, ord), Equal);
    /// assert_eq!(y.cmp_with_order(&z, ord), Greater);
    ///
    /// assert_eq!(z.cmp_with_order(&x, ord), Less);
    /// assert_eq!(z.cmp_with_order(&y, ord), Less);
    /// assert_eq!(z.cmp_with_order(&z, ord), Equal);
    /// ```
    pub fn cmp_with_order(&self, other: &Self, order: Ordering) -> core::cmp::Ordering
    where
        T: Ord,
    {
        use core::cmp::Ordering::*;

        match (unsafe { self.as_ref_with_order(order) }, unsafe {
            other.as_ref_with_order(order)
        }) {
            (Some(l), Some(r)) => l.cmp(r),
            (Some(_), None) => Greater,
            (None, Some(_)) => Less,
            (None, None) => Equal,
        }
    }
}
