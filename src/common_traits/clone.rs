use crate::{states::SOME, ConcurrentOption};
use std::sync::atomic::Ordering;

impl<T: Clone> Clone for ConcurrentOption<T> {
    /// Clones the concurrent option with the [`Relaxed`] ordering.
    ///
    /// In order to clone with a stronger ordering,
    /// you may call [`clone_with_order`] with the desired ordering.
    ///
    /// [`Relaxed`]: std::sync::atomic::Ordering::Relaxed
    /// [`clone_with_order`]: ConcurrentOption::clone_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(42);
    /// let y = x.clone(); // clone with default Relaxed ordering
    /// assert_eq!(x, y);
    ///
    /// let x = ConcurrentOption::some(42);
    /// let y = x.clone_with_order(Ordering::SeqCst).into(); // clone with desired ordering SeqCst
    /// assert_eq!(x, y);
    /// ```
    fn clone(&self) -> Self {
        match self.state.load(Ordering::Relaxed) {
            SOME => {
                let value = unsafe { self.value_ref() }.clone();
                Self::some(value)
            }
            _ => Self::none(),
        }
    }
}
