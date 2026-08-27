use crate::{ConcurrentOption, states::SOME};

impl<T: Clone> Clone for ConcurrentOption<T> {
    /// Clones the concurrent option with the [`Relaxed`] ordering.
    ///
    /// In order to clone with a stronger ordering,
    /// you may call [`clone_with_order`] with the desired ordering.
    ///
    /// [`Relaxed`]: core::sync::atomic::Ordering::Relaxed
    /// [`clone_with_order`]: ConcurrentOption::clone_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use core::sync::atomic::Ordering;
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
        // hold the lock for the entire clone; `as_ref` alone would release it before `x.clone()` runs
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { (*self.value.get()).assume_init_ref() };
                Self::some(x.clone())
            }
            None => Self::none(),
        }
    }
}
