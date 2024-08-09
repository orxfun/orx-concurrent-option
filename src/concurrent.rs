use crate::{states::*, ConcurrentOption};
use std::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    // concurrent state mutation - special

    /// Thread safe method to initiate the value of the option with the given `value`.
    ///
    /// * Returns `true` if the option was `is_none` variant and initiated with the given value.
    /// * It does nothing if the concurrent option is already of `is_some` variant, and returns `false`.
    ///
    /// Note that it is safe to call this method with a shared reference `&self`.
    ///
    /// This method is particularly useful for enabling concurrent read & write operations,
    /// while the value is expected to be initialized only once.
    ///
    /// # Suggestion on Concurrent Read & Write Operations
    ///
    /// Note that this is a one-time-write method which can happen safely while other threads are reading the option.
    /// It is however recommended to call read-methods with a `&self` reference,
    /// such as `as_ref`, using an ordering stronger than `Relaxed`, such as `Acquire` or `SeqCst`,
    /// if one or more threads are expected to call write methods `initialize_if_none` or `initialize_unchecked` concurrently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let inserted = x.initialize_if_none(3.to_string());
    /// assert!(inserted);
    /// assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&3.to_string()));
    ///
    /// let x = ConcurrentOption::some(7.to_string());
    /// let inserted = x.initialize_if_none(3.to_string()); // does nothing
    /// assert!(!inserted);
    /// assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
    /// ```
    ///
    /// A more advanced and useful example is demonstrated below:
    /// * there exist multiple readers checking the value of an optional; they will receive
    ///   * None if the value is not initialized yet;
    ///   * a reference to the value otherwise.
    /// * there exist multiple initializers each trying to set the value of optional;
    ///   * only the first one will succeed,
    ///   * since this is an initialization method, succeeding calls will safely be ignored.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// fn reader(maybe: &ConcurrentOption<String>) {
    ///     let mut is_none_at_least_once = false;
    ///     let mut is_seven_at_least_once = false;
    ///
    ///     for _ in 0..100 {
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///
    ///         let read = maybe.as_ref_with_order(Ordering::Acquire);
    ///
    ///         let is_none = read.is_none();
    ///         let is_seven = read == Some(&7.to_string());
    ///
    ///         assert!(is_none || is_seven);
    ///
    ///         is_none_at_least_once |= is_none;
    ///         is_seven_at_least_once |= is_seven;
    ///     }
    ///
    ///     assert!(is_none_at_least_once && is_seven_at_least_once);
    /// }
    ///
    /// fn initializer(maybe: &ConcurrentOption<String>) {
    ///     for _ in 0..50 {
    ///         // wait for a while to simulate a delay
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///     }
    ///
    ///     let _ = maybe.initialize_if_none(7.to_string());
    ///
    ///     for _ in 0..50 {
    ///         // it is safe to call `initialize_if_none` on Some variant, it will do nothing
    ///         let inserted = maybe.initialize_if_none(1_000_000.to_string());
    ///         assert!(!inserted);
    ///     }
    /// }
    ///
    /// let num_readers = 8;
    /// let num_writers = 8;
    ///
    /// let maybe = ConcurrentOption::<String>::none();
    /// let maybe_ref = &maybe;
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_readers {
    ///         s.spawn(|| reader(maybe_ref));
    ///     }
    ///
    ///     for _ in 0..num_writers {
    ///         s.spawn(|| initializer(maybe_ref));
    ///     }
    /// });
    ///
    /// assert_eq!(maybe.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
    /// ```
    pub fn initialize_if_none(&self, value: T) -> bool {
        match self.mut_handle(NONE, SOME) {
            Some(_handle) => {
                unsafe { &mut *self.value.get() }.write(value);
                true
            }
            None => false,
        }
    }

    /// Thread safe method to initiate the value of the option with the given `value`
    /// **provided that** the concurrent option `is_none` at the point of initializing.
    ///
    /// See [`initialize_if_none`] for checked version.
    ///
    /// [`initialize_if_none`]: ConcurrentOption::initialize_if_none
    ///
    /// Note that it is safe to call this method with a shared reference `&self`
    /// **provided that** the concurrent option `is_none` at the point of initializing.
    ///
    /// This method is particularly useful for enabling concurrent read & write operations,
    /// while the value is expected to be initialized only once.
    ///
    /// # Safety
    ///
    /// This method can be safely called when the concurrent option is guaranteed to be of None variant.
    ///
    /// On the other hand, calling it when the option has a value leads to undefined behavior
    /// due to the following possible data race where we can be reading the value with a `&self` reference,
    /// while at the same time writing with this unsafe method and a `&self` reference.
    ///
    /// # Suggestion on Concurrent Read & Write Operations
    ///
    /// Note that this is a one-time-write method which can happen safely while other threads are reading the option.
    /// It is however recommended to call read-methods with a `&self` reference,
    /// such as `as_ref`, using an ordering stronger than `Relaxed`, such as `Acquire` or `SeqCst`,
    /// if one or more threads are expected to call write methods `initialize_if_none` or `initialize_unchecked` concurrently.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// unsafe { x.initialize_unchecked(3.to_string()) };
    /// assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&3.to_string()));
    ///
    /// #[cfg(not(miri))]
    /// {
    ///     let x = ConcurrentOption::some(7.to_string());
    ///     unsafe { x.initialize_unchecked(3.to_string()) }; // undefined behavior!
    ///     assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&3.to_string()));
    /// }
    /// ```
    ///
    /// A more advanced and useful example is demonstrated below:
    /// * there exist multiple readers checking the value of an optional; they will receive
    ///   * None if the value is not initialized yet;
    ///   * a reference to the value otherwise.
    /// * there exist multiple initializers each trying to set the value of optional;
    ///   * only the first one will succeed,
    ///   * since this is an initialization method, succeeding calls will safely be ignored.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// fn reader(maybe: &ConcurrentOption<String>) {
    ///     let mut is_none_at_least_once = false;
    ///     let mut is_seven_at_least_once = false;
    ///
    ///     for _ in 0..100 {
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///
    ///         let read = maybe.as_ref_with_order(Ordering::Acquire);
    ///
    ///         let is_none = read.is_none();
    ///         let is_seven = read == Some(&7.to_string());
    ///
    ///         assert!(is_none || is_seven);
    ///
    ///         is_none_at_least_once |= is_none;
    ///         is_seven_at_least_once |= is_seven;
    ///     }
    ///
    ///     assert!(is_none_at_least_once && is_seven_at_least_once);
    /// }
    ///
    /// fn unsafe_initializer(maybe: &ConcurrentOption<String>) {
    ///     for _ in 0..50 {
    ///         // wait for a while to simulate a delay
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///     }
    ///
    ///     // we need to make sure to call initialize_unchecked only once
    ///     unsafe { maybe.initialize_unchecked(7.to_string()) };
    /// }
    ///
    /// let num_readers = 8;
    ///
    /// let maybe = ConcurrentOption::<String>::none();
    /// let maybe_ref = &maybe;
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_readers {
    ///         s.spawn(|| reader(maybe_ref));
    ///     }
    ///
    ///     s.spawn(|| unsafe_initializer(maybe_ref));
    /// });
    ///
    /// assert_eq!(maybe.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
    /// ```
    pub unsafe fn initialize_unchecked(&self, value: T) {
        unsafe { &mut *self.value.get() }.write(value);
        self.state.store(SOME, ORDER_STORE);
    }

    /// Maps the reference of the underlying value with the given function `f`.
    ///
    /// Returns
    /// * None if the option is None
    /// * `f(&value)` if the option is Some(value)
    ///
    /// # Concurrency Notes
    ///
    /// Alternatively, one can take an optional reference of the underlying value by `as_ref`
    /// and mapping outside this function.
    ///
    /// However,
    /// * the map operation via `map_ref` guarantees that the underlying value will not be updated before the operation; while
    /// * the alternative approach with `as_ref` is subject to data race if the state of the optional is concurrently being
    /// updated by methods such as `take`.
    ///   * an exception to this is the `initialize_if_none` method which fits very well the initialize-once scenarios;
    /// here, `as_ref` and `initialize_if_none` can safely be called concurrently from multiple threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let len = x.map_ref(|x| x.len());
    /// assert_eq!(len, None);
    ///
    /// let x = ConcurrentOption::some("foo".to_string());
    /// let len = x.map_ref(|x| x.len());
    /// assert_eq!(len, Some(3));
    /// ```
    pub fn map_ref<U, F>(&self, f: F) -> Option<U>
    where
        F: FnOnce(&T) -> U,
    {
        match self.mut_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                let x = unsafe { std::mem::MaybeUninit::assume_init_ref(x) };
                Some(f(x))
            }
            None => None,
        }
    }

    // concurrent state mutation

    /// Thread safe method to take the value out of the option if Some,
    /// leaving a None in its place.
    ///
    /// Has no impact and returns None, if the option is of None variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(42);
    /// let y = x.take();
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert_eq!(y, Some(42));
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y = x.take();
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert_eq!(y, None);
    /// ```
    pub fn take(&self) -> Option<T> {
        match self.mut_handle(SOME, NONE) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                Some(unsafe { std::mem::MaybeUninit::assume_init_read(x) })
            }
            None => None,
        }
    }

    /// Thread safe method to take the value out of the option, but only if the predicate evaluates to
    /// `true` on a mutable reference to the value.
    ///
    /// In other words, replaces `self` with None if the predicate returns `true`.
    /// This method operates similar to [`ConcurrentOption::take`] but conditional.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(42);
    ///
    /// let prev = x.take_if(|v| if *v == 42 {
    ///     *v += 1;
    ///     false
    /// } else {
    ///     false
    /// });
    /// assert_eq!(x, ConcurrentOption::some(43));
    /// assert_eq!(prev, None);
    ///
    /// let prev = x.take_if(|v| *v == 43);
    /// assert_eq!(x, ConcurrentOption::none());
    /// assert_eq!(prev, Some(43));
    /// ```
    pub fn take_if<P>(&self, predicate: P) -> Option<T>
    where
        P: FnOnce(&mut T) -> bool,
        T: std::fmt::Debug,
    {
        match self
            .state
            .compare_exchange(SOME, RESERVED, ORDER_LOAD, ORDER_LOAD)
            .is_ok()
        {
            true => {
                let x = unsafe { &mut *self.value.get() };
                let x_mut = unsafe { std::mem::MaybeUninit::assume_init_mut(x) };
                let output = match predicate(x_mut) {
                    false => None,
                    true => Some(unsafe { std::mem::MaybeUninit::assume_init_read(x) }),
                };

                let success_state = match output.is_some() {
                    true => NONE,
                    false => SOME,
                };
                self.state
                    .compare_exchange(RESERVED, success_state, ORDER_STORE, ORDER_STORE)
                    .expect(
                        "Failed to update the concurrent state after concurrent state mutation",
                    );

                output
            }
            false => None,
        }
    }

    // common traits

    /// Clones the concurrent option with the desired `order` into an Option.
    ///
    /// Note that the `Clone` trait implementation clones the concurrent option with the default ordering.
    ///
    /// You may use `clone_with_order` in order to clone with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let mut x = ConcurrentOption::some(42);
    /// let y = x.clone_with_order(Ordering::SeqCst);
    /// assert_eq!(x.take(), y);
    /// ```
    pub fn clone_with_order(&self, order: Ordering) -> Option<T>
    where
        T: Clone,
    {
        self.as_ref_with_order(order).cloned()
    }

    /// Returns whether or not self is equal to the `other` with the desired `order`.
    ///
    /// Note that the `PartialEq` trait implementation checks equality with the default ordering.
    ///
    /// You may use `eq_with_order` in order to check equality with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::sync::atomic::Ordering;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let o = Ordering::SeqCst;
    ///
    /// assert!(x.eq_with_order(&x, o));
    /// assert!(!x.eq_with_order(&y, o));
    /// assert!(!x.eq_with_order(&z, o));
    ///
    /// assert!(!z.eq_with_order(&x, o));
    /// assert!(!z.eq_with_order(&y, o));
    /// assert!(z.eq_with_order(&z, o));
    /// ```
    pub fn eq_with_order(&self, other: &Self, order: Ordering) -> bool
    where
        T: PartialEq,
    {
        match (
            self.as_ref_with_order(order),
            other.as_ref_with_order(order),
        ) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y),
            _ => false,
        }
    }

    /// Returns an ordering between `self` and `other` with the desired `order`.
    ///
    /// Note that the `PartialOrd` trait implementation checks equality with the default ordering.
    ///
    /// You may use `partial_cmp_with_order` in order to check equality with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let ord = std::sync::atomic::Ordering::SeqCst;
    ///
    /// assert_eq!(x.partial_cmp_with_order(&x, ord), Some(Equal));
    /// assert_eq!(x.partial_cmp_with_order(&y, ord), Some(Less));
    /// assert_eq!(x.partial_cmp_with_order(&z, ord), Some(Greater));
    ///
    /// assert_eq!(y.partial_cmp_with_order(&x, ord), Some(Greater));
    /// assert_eq!(y.partial_cmp_with_order(&y, ord), Some(Equal));
    /// assert_eq!(y.partial_cmp_with_order(&z, ord), Some(Greater));
    ///
    /// assert_eq!(z.partial_cmp_with_order(&x, ord), Some(Less));
    /// assert_eq!(z.partial_cmp_with_order(&y, ord), Some(Less));
    /// assert_eq!(z.partial_cmp_with_order(&z, ord), Some(Equal));
    /// ```
    pub fn partial_cmp_with_order(
        &self,
        other: &Self,
        order: Ordering,
    ) -> Option<std::cmp::Ordering>
    where
        T: PartialOrd,
    {
        use std::cmp::Ordering::*;

        match (
            self.as_ref_with_order(order),
            other.as_ref_with_order(order),
        ) {
            (Some(l), Some(r)) => l.partial_cmp(r),
            (Some(_), None) => Some(Greater),
            (None, Some(_)) => Some(Less),
            (None, None) => Some(Equal),
        }
    }

    /// Returns an ordering between `self` and `other` with the desired `order`.
    ///
    /// Note that the `Ord` trait implementation checks equality with the default ordering.
    ///
    /// You may use `cmp_with_order` in order to check equality with the desired ordering.
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    /// use std::cmp::Ordering::*;
    ///
    /// let x = ConcurrentOption::some(3);
    /// let y = ConcurrentOption::some(7);
    /// let z = ConcurrentOption::<i32>::none();
    ///
    /// let ord = std::sync::atomic::Ordering::SeqCst;
    ///
    /// assert_eq!(x.cmp_with_order(&x, ord), Equal);
    /// assert_eq!(x.cmp_with_order(&y, ord), Less);
    /// assert_eq!(x.cmp_with_order(&z, ord), Greater);
    ///
    /// assert_eq!(y.cmp_with_order(&x, ord), Greater);
    /// assert_eq!(y.cmp_with_order(&y, ord), Equal);
    /// assert_eq!(y.cmp_with_order(&z, ord), Greater);
    ///
    /// assert_eq!(z.cmp_with_order(&x, ord), Less);
    /// assert_eq!(z.cmp_with_order(&y, ord), Less);
    /// assert_eq!(z.cmp_with_order(&z, ord), Equal);
    /// ```
    pub fn cmp_with_order(&self, other: &Self, order: Ordering) -> std::cmp::Ordering
    where
        T: Ord,
    {
        use std::cmp::Ordering::*;

        match (
            self.as_ref_with_order(order),
            other.as_ref_with_order(order),
        ) {
            (Some(l), Some(r)) => l.cmp(r),
            (Some(_), None) => Greater,
            (None, Some(_)) => Less,
            (None, None) => Equal,
        }
    }
}
