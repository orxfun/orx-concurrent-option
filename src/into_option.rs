use crate::ConcurrentOption;

/// Trait representing types that can be converted into a standard Option.
///
/// # Examples
///
/// ```rust
/// use orx_concurrent_option::*;
///
/// let con_option: ConcurrentOption<i32> = ConcurrentOption::some(42);
/// assert_eq!(con_option.into_option(), Some(42));
///
/// let option: Option<i32> = Some(42);
/// assert_eq!(option.into_option(), Some(42));
/// ```
pub trait IntoOption<T> {
    /// Converts self into Option.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let con_option: ConcurrentOption<i32> = ConcurrentOption::some(42);
    /// assert_eq!(con_option.into_option(), Some(42));
    ///
    /// let option: Option<i32> = Some(42);
    /// assert_eq!(option.into_option(), Some(42));
    /// ```
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T> IntoOption<T> for ConcurrentOption<T> {
    fn into_option(mut self) -> Option<T> {
        self.exclusive_take()
    }
}
