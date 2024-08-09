use crate::{states::*, ConcurrentOption};
use std::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    /// Returns:
    /// * a raw `*const T` pointer to the underlying data when the option is of Some variant;
    /// * `None` otherwise.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.raw_get(Ordering::SeqCst);
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.raw_get(Ordering::Acquire);
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    /// ```
    pub fn raw_get(&self, order: Ordering) -> Option<*const T> {
        match self.state.load(order) {
            SOME => {
                let x = unsafe { &*self.value.get() };
                Some(x.as_ptr())
            }
            _ => None,
        }
    }

    /// Returns:
    /// * a raw `*mut T` pointer to the underlying data when the option is of Some variant;
    /// * `None` otherwise.
    ///
    /// Depending on requirement of the use case, `Relaxed`, `Acquire` or `SeqCst` can be used as the `order`.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.raw_get_mut(Ordering::SeqCst);
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.raw_get_mut(Ordering::Acquire);
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.raw_get(Ordering::Relaxed).unwrap();
    /// assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.raw_get_mut(Ordering::Relaxed);
    /// let p = p.unwrap();
    /// let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    /// assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
    /// ```
    pub fn raw_get_mut(&self, order: Ordering) -> Option<*mut T> {
        match self.state.load(order) {
            SOME => {
                let x = unsafe { &mut *self.value.get() };
                Some(x.as_mut_ptr())
            }
            _ => None,
        }
    }
}
