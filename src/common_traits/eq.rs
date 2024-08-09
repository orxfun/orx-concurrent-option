use crate::concurrent_option::ConcurrentOption;

impl<T: PartialEq> PartialEq for ConcurrentOption<T> {
    /// Returns whether or not self is equal to the `other` with the default ordering.
    ///
    /// You may call [`eq_with_order`] to use the desired ordering.
    ///
    /// [`eq_with_order`]: ConcurrentOption::eq_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// assert!(x.eq(&x));
    /// assert!(!x.eq(&y));
    /// assert!(!x.eq(&z));
    ///
    /// assert!(!z.eq(&x));
    /// assert!(!z.eq(&y));
    /// assert!(z.eq(&z));
    /// ```
    fn eq(&self, other: &Self) -> bool {
        match unsafe { (self.as_ref(), other.as_ref()) } {
            (Some(l), Some(r)) => l.eq(r),
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }
}

impl<T: Eq> Eq for ConcurrentOption<T> {}
