use crate::{states::*, ConcurrentOption};
use core::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    // raw

    /// Returns:
    /// * a raw `*const T` pointer to the underlying data when the option is of Some variant;
    /// * `None` otherwise.
    ///
    /// See [`get_raw_with_order`] to explicitly set the ordering.
    ///
    /// [`get_raw_with_order`]: ConcurrentOption::get_raw_with_order
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.get_raw();
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.get_raw();
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    /// ```
    pub fn get_raw(&self) -> Option<*const T> {
        match self.spin_get_handle(SOME, SOME) {
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
    /// See [`get_raw_mut_with_order`] to explicitly set the ordering.
    ///
    /// [`get_raw_mut_with_order`]: ConcurrentOption::get_raw_mut_with_order
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.get_raw_mut();
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.get_raw_mut();
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.get_raw().unwrap();
    /// assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.get_raw_mut();
    /// let p = p.unwrap();
    /// let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    /// assert_eq!(unsafe { x.as_ref() }, Some(&7.to_string()));
    /// ```
    pub fn get_raw_mut(&self) -> Option<*mut T> {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &mut *self.value.get() };
                Some(x.as_mut_ptr())
            }
            None => None,
        }
    }

    // raw with-order

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
    /// use core::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.get_raw_with_order(Ordering::SeqCst);
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.get_raw_with_order(Ordering::Acquire);
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    /// ```
    pub fn get_raw_with_order(&self, order: Ordering) -> Option<*const T> {
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
    /// use core::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let p = x.get_raw_mut_with_order(Ordering::SeqCst);
    /// assert!(p.is_none());
    ///
    /// let x = ConcurrentOption::some(3.to_string());
    /// let p = x.get_raw_mut_with_order(Ordering::Acquire);
    /// assert!(p.is_some());
    /// assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.get_raw_mut_with_order(Ordering::Relaxed).unwrap();
    /// assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));
    ///
    /// let p = x.get_raw_mut_with_order(Ordering::Relaxed);
    /// let p = p.unwrap();
    /// let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    /// assert_eq!(unsafe { x.as_ref() }, Some(&7.to_string()));
    /// ```
    pub fn get_raw_mut_with_order(&self, order: Ordering) -> Option<*mut T> {
        match self.state.load(order) {
            SOME => {
                let x = unsafe { &mut *self.value.get() };
                Some(x.as_mut_ptr())
            }
            _ => None,
        }
    }
}
