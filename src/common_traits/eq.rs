use crate::concurrent_option::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T: PartialEq> PartialEq for ConcurrentOption<T> {
    /// Returns whether or not self is equal to the `other` the [`Relaxed`] ordering.
    ///
    /// In order to compare with a stronger ordering,
    /// you may call [`eq_with_order`] with the desired ordering.
    ///
    /// [`Relaxed`]: std::sync::atomic::Ordering::Relaxed
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
        self.eq_with_order(other, Ordering::Relaxed)
    }
}

impl<T: Eq> Eq for ConcurrentOption<T> {}
