use crate::{handle::Handle, mut_handle::MutHandle, states::StateU8};
use core::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicU8};

/// ConcurrentOption is a thread-safe and lock-free read-write option type.
///
/// ## ConcurrentOption Methods In Groups
///
/// ConcurrentOption methods are based on the standard Option with minor differences in order to better fit concurrent programs.
///
/// For example, instead of `fn map<U, F>(self, f: F) -> Option<U>`
/// * ConcurrentOption implements `fn map<U, F>(&self, f: F) -> Option<U>` which is specialized to map over the reference while guaranteeing the lack of data race.
/// * Note that the prior result can trivially be obtained by `maybe.exclusive_take().map(f)` when we have the ownership.
///
/// ### ⬤ Methods requiring self or &mut self
///
/// These methods are safe by the borrow checker and they behave similar to the original variants.
///
/// In order to separate them from the thread-safe versions, methods requiring `&mut self` are prefixed with **exclusive_**.
///
/// Some such methods are `unwrap`, `expect`, `exclusive_mut` or `exclusive_take`.
///
/// ### ⬤ Thread safe versions of mutating methods
///
/// Thread safe variants of mutating methods are available and they can be safely be called with a shared `&self` reference.
///
/// Some examples are `take`, `take_if`, `replace`, etc.
///
/// These methods guarantee that there exist no other mutation or no reading during the mutation.
///
/// ### ⬤ Thread safe versions of read methods
///
/// Thread safe variants of methods which access the underlying value to calculate a result are available.
///
/// Some examples are `is_some`, `map`, `and_then`, etc.
///
/// These methods guarantee that there exist no mutation while reading the data.
///
/// ### ⬤ Partially thread safe methods
///
/// Methods which return a shared reference `&T` or mutable reference `&mut T` to the underlying value of the optional are marked as `unsafe`.
///
/// These methods internally guarantee the creation of a valid reference in the absence of a data race. In this sense, they are thread safe.
///
/// On the other hand, since they return the reference, the reference is leaked outside the type. A succeeding mutation might lead to a data race, and hence, to an undefined behavior.
///
/// Some example methods are `as_ref`, `as_deref`, `insert`, etc.
///
/// ### ⬤ Methods to allow manual control on concurrency
///
/// ConcurrentOption also exposes methods which accepts a `core::sync::atomic::Ordering` and gives the control to the caller. These methods are suffixed with **with_order**, except for the state.
///
/// Some such methods are `state`, `as_ref_with_order`, `get_raw_with_order`, `clone_with_order`, etc.
///
/// ## Examples
///
/// ### Concurrent Read & Write
///
/// The following example demonstrates the ease of concurrently mutating the state of the option while safely reading the underlying data with multiple reader and writer threads.
///
/// ```rust
/// use orx_concurrent_option::*;
/// use std::time::Duration;
///
/// enum MutOperation {
///     InitializeIfNone,
///     UpdateIfSome,
///     Replace,
///     Take,
///     TakeIf,
/// }
///
/// impl MutOperation {
///     fn new(i: usize) -> Self {
///         match i % 5 {
///             0 => Self::InitializeIfNone,
///             1 => Self::UpdateIfSome,
///             2 => Self::Replace,
///             3 => Self::Take,
///             _ => Self::TakeIf,
///         }
///     }
/// }
///
/// let num_readers = 8;
/// let num_writers = 8;
///
/// let values = vec![ConcurrentOption::<String>::none(); 8];
///
/// std::thread::scope(|s| {
///     for _ in 0..num_readers {
///         s.spawn(|| {
///             for _ in 0..100 {
///                 std::thread::sleep(Duration::from_millis(100));
///                 let mut num_chars = 0;
///                 for maybe in &values {
///                     // concurrently access the value
///                     num_chars += maybe.map(|x| x.len()).unwrap_or(0);
///                 }
///                 assert!(num_chars <= 100);
///             }
///         });
///     }
///
///     for _ in 0..num_writers {
///         s.spawn(|| {
///             for i in 0..100 {
///                 std::thread::sleep(Duration::from_millis(100));
///                 let e = i % values.len();
///
///                 // concurrently update the option
///                 match MutOperation::new(i) {
///                     MutOperation::InitializeIfNone => {
///                         values[e].initialize_if_none(e.to_string());
///                     }
///                     MutOperation::UpdateIfSome => {
///                         values[e].update_if_some(|x| *x = format!("{}!", x));
///                     }
///                     MutOperation::Replace => {
///                         values[e].replace(e.to_string());
///                     }
///                     MutOperation::Take => {
///                         _ = values[e].take();
///                     }
///                     MutOperation::TakeIf => _ = values[e].take_if(|x| x.len() < 2),
///                 }
///                 let e = i % values.len();
///                 _ = values[e].initialize_if_none(e.to_string());
///             }
///         });
///     }
/// })
/// ```
///
/// ### Concurrent Initialize & Read
///
/// A common use case for option is to model a delayed initialization; rather than concurrent mutation. In other words, we start with a None variant and at some point we receive the value and convert our option to Some(value), which will then stay as Some(value) throughout its lifetime.
///
/// This scenario demonstrates a use case where we can safely leak a reference outside the optional:
/// * All references provided by ConcurrentOption are valid and data race free at the point they are obtained. In other words, we can only obtain a reference after the value is initialized; i.e., the option becomes Some(value).
/// * Since we will never mutate the option after initialization, we can safely keep a reference to it without a concern about a data race.
///   * However, no further mutation is our promise and responsibility as the caller. ConcurrentOption has no control over the leaked references; and hence, obtaining the reference is through the unsafe `as_ref` method.
///
/// For this scenario, we can make use of two matching methods:
/// * `initialize_if_none` is a thread safe method to initialize the value of the option to the given value. It is safe to call the method on a Some variant, it will have no impact. Further, it makes sure that no reader can access the value until it is completely initialized.
/// * `as_ref` method returns a reference to the underlying value if the option is a Some variant. Otherwise, if the value has not been initialized, we will safely receive None. Note that we could also use `as_ref_with_order` paired up with `Acquire` or `SeqCst` ordering if we want to model the access ordering manually.
///
/// ```rust
/// use orx_concurrent_option::*;
///
/// fn reader(maybe: &ConcurrentOption<String>) {
///     let mut is_none_at_least_once = false;
///     let mut is_seven_at_least_once = false;
///     for _ in 0..100 {
///         std::thread::sleep(std::time::Duration::from_millis(100));
///
///         let read = unsafe { maybe.as_ref() };
///         let is_none = read.is_none();
///         let is_seven = read == Some(&7.to_string());
///
///         assert!(is_none || is_seven);
///
///         is_none_at_least_once |= is_none;
///         is_seven_at_least_once |= is_seven;
///     }
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
///         // it is safe to call `initialize_if_none` on Some variant
///         // it will do nothing
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
///     for _ in 0..num_writers {
///         s.spawn(|| initializer(maybe_ref));
///     }
/// });
///
/// assert_eq!(maybe.unwrap(), 7.to_string());
/// ```
pub struct ConcurrentOption<T> {
    pub(crate) value: UnsafeCell<MaybeUninit<T>>,
    pub(crate) state: AtomicU8,
}

impl<T> ConcurrentOption<T> {
    pub(crate) fn get_handle(
        &self,
        initial_state: StateU8,
        success_state: StateU8,
    ) -> Option<Handle<'_>> {
        Handle::get(&self.state, initial_state, success_state)
    }

    #[inline(always)]
    pub(crate) fn spin_get_handle(
        &self,
        initial_state: StateU8,
        success_state: StateU8,
    ) -> Option<Handle<'_>> {
        Handle::spin_get(&self.state, initial_state, success_state)
    }

    /// Provides the mut handle on the value of the optional:
    /// * the optional must be in the `initial_state` for this method to succeed,
    /// * the optional will be brought to `success_state` once the handle is dropped.
    ///
    /// # Safety
    ///
    /// This method is unsafe since the handle provides direct access to the underlying
    /// value, skipping thread-safety guarantees.
    pub unsafe fn mut_handle(
        &self,
        initial_state: StateU8,
        success_state: StateU8,
    ) -> Option<MutHandle<T>> {
        MutHandle::spin_get(self, initial_state, success_state)
    }
}

unsafe impl<T: Send> Send for ConcurrentOption<T> {}

unsafe impl<T: Sync> Sync for ConcurrentOption<T> {}
