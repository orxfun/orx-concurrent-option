use crate::{states::*, ConcurrentOption};
use std::{ops::Deref, sync::atomic::Ordering};

impl<T> ConcurrentOption<T> {
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
        self.state.load(order) != SOME
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

    /// Clones the concurrent option with the desired `order` into an Option.
    ///
    /// Note that the `Clone` trait implementation clones the concurrent option with the default ordering.
    ///
    /// You may use `clone_with_order` in order to clone with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
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
    /// use std::sync::atomic::Ordering;
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
    /// use std::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let ord = std::sync::atomic::Ordering::SeqCst;
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
    ) -> Option<std::cmp::Ordering>
    where
        T: PartialOrd,
    {
        use std::cmp::Ordering::*;

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
    /// use std::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let ord = std::sync::atomic::Ordering::SeqCst;
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
    pub fn cmp_with_order(&self, other: &Self, order: Ordering) -> std::cmp::Ordering
    where
        T: Ord,
    {
        use std::cmp::Ordering::*;

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
