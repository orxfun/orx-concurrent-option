use crate::{states::*, ConcurrentOption};
use std::mem::MaybeUninit;

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
    ///
    /// let x = ConcurrentOption::<String>::none();
    /// let inserted = x.initialize_if_none(3.to_string());
    /// assert!(inserted);
    /// assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));
    ///
    /// let x = ConcurrentOption::some(7.to_string());
    /// let inserted = x.initialize_if_none(3.to_string()); // does nothing
    /// assert!(!inserted);
    /// assert_eq!(unsafe { x.as_ref() }, Some(&7.to_string()));
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
    ///
    /// fn reader(maybe: &ConcurrentOption<String>) {
    ///     let mut is_none_at_least_once = false;
    ///     let mut is_seven_at_least_once = false;
    ///
    ///     for _ in 0..100 {
    ///         std::thread::sleep(std::time::Duration::from_millis(100));
    ///
    ///         let read = unsafe { maybe.as_ref() };
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
    /// assert_eq!(maybe.unwrap(), 7.to_string());
    /// ```
    pub fn initialize_if_none(&self, value: T) -> bool {
        match self.get_handle(NONE, SOME) {
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
    /// assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));
    ///
    /// #[cfg(not(miri))]
    /// {
    ///     let x = ConcurrentOption::some(7.to_string());
    ///     unsafe { x.initialize_unchecked(3.to_string()) }; // undefined behavior!
    ///     assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));
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
    ///         let read = unsafe { maybe.as_ref() };
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
    /// assert_eq!(maybe.unwrap(), 7.to_string());
    /// ```
    pub unsafe fn initialize_unchecked(&self, value: T) {
        unsafe { &mut *self.value.get() }.write(value);
        self.state.store(SOME, ORDER_STORE);
    }

    // concurrent state mutation

    /// Thread safe method to update the value of the option if it is of Some variant.
    /// Does nothing if it is None.
    ///
    /// # Example
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let maybe = ConcurrentOption::<String>::none();
    /// maybe.update_if_some(|x| *x = format!("{}!", x));
    /// assert!(maybe.is_none());
    ///
    /// let maybe = ConcurrentOption::some(42.to_string());
    /// maybe.update_if_some(|x| *x = format!("{}!", x));
    /// assert!(maybe.is_some_and(|x| x == &"42!".to_string()));
    /// ```
    pub fn update_if_some<F>(&self, mut f: F) -> bool
    where
        F: FnMut(&mut T),
    {
        match self.spin_get_handle(SOME, SOME) {
            Some(_handle) => {
                let x = unsafe { &mut *self.value.get() };
                let x = unsafe { MaybeUninit::assume_init_mut(x) };
                f(x);
                true
            }
            None => false,
        };
        true
    }

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
        match self.spin_get_handle(SOME, NONE) {
            Some(_handle) => {
                let x = unsafe { &*self.value.get() };
                Some(unsafe { MaybeUninit::assume_init_read(x) })
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
    #[allow(clippy::missing_panics_doc, clippy::unwrap_in_result)]
    pub fn take_if<P>(&self, predicate: P) -> Option<T>
    where
        P: FnOnce(&mut T) -> bool,
    {
        loop {
            match self
                .state
                .compare_exchange(SOME, RESERVED, ORDER_LOAD, ORDER_LOAD)
            {
                Ok(_) => {
                    let x = unsafe { &mut *self.value.get() };
                    let x_mut = unsafe { MaybeUninit::assume_init_mut(x) };
                    let output = match predicate(x_mut) {
                        false => None,
                        true => Some(unsafe { MaybeUninit::assume_init_read(x) }),
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

                    return output;
                }
                Err(previous_state) => match previous_state {
                    RESERVED => continue,
                    _ => return None,
                },
            }
        }
    }

    /// Thread safe method to replace the actual value in the option by the value given in parameter,
    /// returning the old value if present,
    /// leaving a Some in its place without de-initializing either one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some(2);
    /// let old = x.replace(5);
    /// assert_eq!(x, ConcurrentOption::some(5));
    /// assert_eq!(old, Some(2));
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let old = x.replace(3);
    /// assert_eq!(x, ConcurrentOption::some(3));
    /// assert_eq!(old, None);
    /// ```
    pub fn replace(&self, value: T) -> Option<T> {
        loop {
            if let Some(_handle) = self.spin_get_handle(SOME, SOME) {
                let x = unsafe { (*self.value.get()).assume_init_mut() };
                let old = std::mem::replace(x, value);
                return Some(old);
            }

            if let Some(_handle) = self.spin_get_handle(NONE, SOME) {
                let x = unsafe { &mut *self.value.get() };
                x.write(value);
                return None;
            }
        }
    }

    /// Partially thread safe method to insert `value` into the option, and then to return a mutable reference to it.
    ///
    /// If the option already contains a value, the old value is dropped.
    ///
    /// See also [`Option::get_or_insert`], which doesn't update the value if
    /// the option already contains Some.
    ///
    /// # Safety
    ///
    /// Note that the insertion part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned mutable reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned mutable reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent reads or writes while mutating the value.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let opt: ConcurrentOption<_> = ConcurrentOption::none();
    ///
    /// let val = unsafe { opt.insert(1) };
    /// assert_eq!(*val, 1);
    /// assert_eq!(unsafe { opt.as_ref() }, Some(&1));
    ///
    /// let val = unsafe { opt.insert(2) };
    /// assert_eq!(*val, 2);
    /// *val = 3;
    /// assert_eq!(opt.unwrap(), 3);
    /// ```
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn insert(&self, value: T) -> &mut T {
        loop {
            if let Some(_handle) = self.spin_get_handle(SOME, SOME) {
                let x = unsafe { (*self.value.get()).assume_init_mut() };
                let _old = std::mem::replace(x, value);
                return x;
            }

            if let Some(_handle) = self.spin_get_handle(NONE, SOME) {
                let x = unsafe { &mut *self.value.get() };
                x.write(value);
                return unsafe { x.assume_init_mut() };
            }
        }
    }

    /// Inserts `value` into the option if it is None, then
    /// returns a mutable reference to the contained value.
    ///
    /// See also [`ConcurrentOption::insert`], which updates the value even if
    /// the option already contains Some.
    ///
    /// # Safety
    ///
    /// Note that the insertion part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned mutable reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned mutable reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent reads or writes while mutating the value.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::none();
    ///
    /// {
    ///     let y: &mut u32 = unsafe { x.get_or_insert(5) };
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, ConcurrentOption::some(7));
    /// ```
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_or_insert(&self, value: T) -> &mut T {
        self.get_or_insert_with(|| value)
    }

    /// Partially thread safe method to insert a value computed from `f` into the option if it is None,
    /// then returns a mutable reference to the contained value.
    ///
    /// # Safety
    ///
    /// Note that the insertion part of this method is thread safe.
    ///
    /// The method is `unsafe` due to the returned mutable reference to the underlying value.
    ///
    /// * It is safe to use this method if the returned mutable reference is discarded (miri would still complain).
    /// * It is also safe to use this method if the caller is able to guarantee that there exist
    /// no concurrent reads or writes while mutating the value.
    /// * Otherwise, it will lead to an **Undefined Behavior** due to data race.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::none();
    ///
    /// {
    ///     let y: &mut u32 = unsafe { x.get_or_insert_with(|| 5) };
    ///     assert_eq!(y, &5);
    ///
    ///     *y = 7;
    /// }
    ///
    /// assert_eq!(x, ConcurrentOption::some(7));
    /// ```
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_or_insert_with<F>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        loop {
            if let Some(_handle) = self.spin_get_handle(SOME, SOME) {
                return unsafe { (*self.value.get()).assume_init_mut() };
            }

            if let Some(_handle) = self.spin_get_handle(NONE, SOME) {
                let x = unsafe { &mut *self.value.get() };
                x.write(f());
                return unsafe { x.assume_init_mut() };
            }
        }
    }
}
