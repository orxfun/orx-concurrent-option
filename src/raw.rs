use crate::{states::*, ConcurrentOption};
use std::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    // row

    /// Returns:
    /// * a raw `*const T` pointer to the underlying data when the option is of Some variant;
    /// * `None` otherwise.
    ///
    /// See [`raw_get_with_order`] to explicitly set the ordering.
    ///
    /// [`raw_get_with_order`]: ConcurrentOption::raw_get_with_order
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.raw_get();
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.raw_get();
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    /// ```
    pub fn raw_get(&self) -> Option<*const T> {
        match self.mut_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                Some(x.as_ptr())
            }
            None => None,
        }
    }

    /// Returns:
    /// * a raw `*mut T` pointer to the underlying data when the option is of Some variant;
    /// * `None` otherwise.
    ///
    /// See [`raw_get_mut_with_order`] to explicitly set the ordering.
    ///
    /// [`raw_get_mut_with_order`]: ConcurrentOption::raw_get_mut_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.raw_get_mut();
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.raw_get_mut();
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.raw_get().unwrap();
    /// assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.raw_get_mut();
    /// let p = p.unwrap();
    /// let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    /// assert_eq!(x.as_ref(), Some(&7.to_string()));
    /// ```
    pub fn raw_get_mut(&self) -> Option<*mut T> {
        match self.mut_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &mut *self.value.get() };
                Some(x.as_mut_ptr())
            }
            None => None,
        }
    }

    // row with-order

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
    /// let p = x.raw_get_with_order(Ordering::SeqCst);
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.raw_get_with_order(Ordering::Acquire);
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    /// ```
    pub fn raw_get_with_order(&self, order: Ordering) -> Option<*const T> {
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
    /// let p = x.raw_get_mut_with_order(Ordering::SeqCst);
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.raw_get_mut_with_order(Ordering::Acquire);
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.raw_get_mut_with_order(Ordering::Relaxed).unwrap();
    /// assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.raw_get_mut_with_order(Ordering::Relaxed);
    /// let p = p.unwrap();
    /// let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    /// assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
    /// ```
    pub fn raw_get_mut_with_order(&self, order: Ordering) -> Option<*mut T> {
        match self.state.load(order) {
            SOME => {
                let x = unsafe { &mut *self.value.get() };
                Some(x.as_mut_ptr())
            }
            _ => None,
        }
    }
}
