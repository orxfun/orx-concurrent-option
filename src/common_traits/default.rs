use crate::ConcurrentOption;

impl<T> Default for ConcurrentOption<T> {
    /// Returns the default value of `ConcurrentOption`, which is `Concurrent::none()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption::<String> = Default::default();
    /// assert_eq!(x, ConcurrentOption::none());
    /// ```
    fn default() -> Self {
        Self::none()
    }
}
