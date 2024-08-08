use crate::ConcurrentOption;

// FROM

impl<T> From<T> for ConcurrentOption<T> {
    /// Wraps the existing value to a `ConcurrentOption` of Some variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<String> = 3.to_string().into();
    /// assert_eq!(x.as_ref(Ordering::Relaxed), Some(&3.to_string()));
    /// ```
    fn from(value: T) -> Self {
        ConcurrentOption::some(value)
    }
}

impl<T> From<Option<T>> for ConcurrentOption<T> {
    /// Converts an `Option` to a `ConcurrentOption`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x: ConcurrentOption<String> = Some(3.to_string()).into();
    /// assert_eq!(x.as_ref(Ordering::Relaxed), Some(&3.to_string()));
    ///
    /// let x: ConcurrentOption<String> = None.into();
    /// assert_eq!(x.as_ref(Ordering::Relaxed), None);
    /// ```
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => ConcurrentOption::some(value),
            None => ConcurrentOption::none(),
        }
    }
}

// INTO

impl<T> From<ConcurrentOption<T>> for Option<T> {
    /// Converts a `ConcurrentOption` to a `Option`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let y: Option<_> = x.into();
    /// assert_eq!(y, Some(3.to_string()));
    ///
    /// let x: ConcurrentOption<String> = ConcurrentOption::none();
    /// let y: Option<String> = x.into();
    /// assert_eq!(y, None);
    /// ```
    fn from(mut value: ConcurrentOption<T>) -> Self {
        value.take()
    }
}
