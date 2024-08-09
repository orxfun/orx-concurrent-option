use crate::concurrent_option::ConcurrentOption;
use crate::states::*;
use std::mem::MaybeUninit;

impl<T> ConcurrentOption<T> {
    /// Creates a concurrent option of the Some variant with an existing value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// assert_eq!(x, ConcurrentOption::some(3.to_string()));
    /// assert_ne!(x, ConcurrentOption::none());
    ///
    /// assert!(x.is_some());
    /// assert!(!x.is_none());
    /// ```
    pub fn some(value: T) -> Self {
        Self {
            value: MaybeUninit::new(value).into(),
            state: SOME.into(),
        }
    }

    /// Creates a concurrent option of the None variant with a missing value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// assert_ne!(x, ConcurrentOption::some(3.to_string()));
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert!(!x.is_some());
    /// assert!(x.is_none());
    ///
    /// let x = ConcurrentOption::default();
    /// assert_ne!(x, ConcurrentOption::some(3.to_string()));
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert!(!x.is_some());
    /// assert!(x.is_none());
    /// ```
    pub fn none() -> Self {
        let value = MaybeUninit::uninit();
        let value = unsafe { value.assume_init() };
        Self {
            value,
            state: NONE.into(),
        }
    }
}
