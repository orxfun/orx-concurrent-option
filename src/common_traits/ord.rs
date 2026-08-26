use crate::ConcurrentOption;
use core::cmp::Ordering::*;

impl<T: PartialOrd> PartialOrd for ConcurrentOption<T> {
    /// Returns an ordering between `self` and `other` with the default ordering.
    ///
    /// You may call [`partial_cmp_with_order`] to use the desired ordering.
    ///
    /// [`partial_cmp_with_order`]: ConcurrentOption::partial_cmp_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// assert_eq!(x.partial_cmp(&x), Some(Equal));
    /// assert_eq!(x.partial_cmp(&y), Some(Less));
    /// assert_eq!(x.partial_cmp(&z), Some(Greater));
    ///
    /// assert_eq!(y.partial_cmp(&x), Some(Greater));
    /// assert_eq!(y.partial_cmp(&y), Some(Equal));
    /// assert_eq!(y.partial_cmp(&z), Some(Greater));
    ///
    /// assert_eq!(z.partial_cmp(&x), Some(Less));
    /// assert_eq!(z.partial_cmp(&y), Some(Less));
    /// assert_eq!(z.partial_cmp(&z), Some(Equal));
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.locked_compare(other, Some(Equal), Some(Greater), Some(Less), |l, r| {
            l.partial_cmp(r)
        })
    }
}

impl<T: Ord> Ord for ConcurrentOption<T> {
    /// Returns an ordering between `self` and `other` with the default ordering.
    ///
    /// You may call [`cmp_with_order`] to use the desired ordering.
    ///
    /// [`cmp_with_order`]: ConcurrentOption::cmp_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// assert_eq!(x.cmp(&x), Equal);
    /// assert_eq!(x.cmp(&y), Less);
    /// assert_eq!(x.cmp(&z), Greater);
    ///
    /// assert_eq!(y.cmp(&x), Greater);
    /// assert_eq!(y.cmp(&y), Equal);
    /// assert_eq!(y.cmp(&z), Greater);
    ///
    /// assert_eq!(z.cmp(&x), Less);
    /// assert_eq!(z.cmp(&y), Less);
    /// assert_eq!(z.cmp(&z), Equal);
    /// ```
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.locked_compare(other, Equal, Greater, Less, |l, r| l.cmp(r))
    }
}
