use crate::{concurrent_option::ConcurrentOption, states::SOME};
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
        // hold the lock while formatting; `as_ref` alone would release it before the value is read
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { (*self.value.get()).assume_init_ref() };
                write!(f, "Concurrent{:?}", Some(x))
            }
            None => write!(f, "ConcurrentNone"),
        }
    }
}
