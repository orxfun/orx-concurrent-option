use crate::concurrent_option::ConcurrentOption;
use std::{fmt::Debug, sync::atomic::Ordering};

impl<T: Debug> Debug for ConcurrentOption<T> {
    /// Creates the debug representation with the [`Relaxed`] ordering.
    ///
    /// [`Relaxed`]: std::sync::atomic::Ordering::Relaxed
    ///
    /// In order to clone with a stronger ordering,
    /// you may debug after calling `as_ref` with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let y = format!("{:?}", x); // debug with default Relaxed ordering
    /// assert_eq!(y, "ConcurrentSome(\"3\")");
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let y = format!("{:?}", x);
    /// assert_eq!(y, "ConcurrentNone");
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let y = format!("{:?}", x.as_ref(Ordering::Acquire)); // clone with desired ordering Acquire
    /// assert_eq!(y, "Some(\"3\")");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.written.load(Ordering::Relaxed) {
            true => write!(f, "ConcurrentSome({:?})", unsafe { self.value_ref() }),
            false => write!(f, "ConcurrentNone"),
        }
    }
}
