use crate::concurrent_option::ConcurrentOption;
use core::fmt::Debug;

impl<T: Debug> Debug for ConcurrentOption<T> {
    /// Creates the debug representation.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let y = format!("{:?}", x); // debug with default Relaxed ordering
    /// assert_eq!(y, "ConcurrentSome(\"3\")");
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let y = format!("{:?}", x);
    /// assert_eq!(y, "ConcurrentNone");
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let maybe = unsafe { self.as_ref() };
        write!(f, "Concurrent{:?}", maybe)
    }
}
