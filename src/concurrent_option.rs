use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};

/// A lock-free concurrent option type which enables to safely initialize and read the data concurrently.
///
/// ## ConcurrentOption Methods In Four Groups
///
/// ### 1. Methods requiring `&mut self` or `self`
///
/// Since these methods guarantee a unique access to the value, they are identical to those of the standard `Option`.
///
/// Some such example methods are: `unwrap`, `take`, `insert`, etc.
///
/// ### 2. Methods specialized for concurrent writing
///
/// These methods require a shared `&self` reference to update the state of the option concurrently.
///
/// Currently available methods are: `initialize_if_none` and `initialize_unchecked`.
///
/// See the example usage below.
///
/// ### 3. Read methods requiring shared `&self` reference
///
/// These methods are similar to those of the standard `Option` except that they require an additional `Ordering` argument.
///
/// Some such example methods are: `is_some`, `is_none`, `as_ref`, `iter`, etc.
///
/// Additional ordering is required since it is possible that the state of the option is being updated concurrently by specialized methods discussed above.
///
/// * `Ordering::Relaxed` can be used when the state of the option is not being changed concurrently.
/// * However, it is recommended to use a stronger ordering such as `Ordering::Acquire` or `Ordering::SeqCst` when it is possible that the state of the option is being updated concurrently.
///
/// ### 4. Variants of the Common Trait Implementations
///
/// `ConcurrentOption` implements common useful traits such as `Clone`, `PartialEq` or `PartialOrd`. Underlying implementations use the `Ordering::Relaxed` ordering while accessing the value of the option.
///
/// However, in a concurrent setting, we might require to use a stronger ordering. Therefore, corresponding trait methods have variants ending with **_with_order** suffix and accepting an order as the argument to allow the caller to decide on the ordering.
///
/// Some such example methods are: `clone_with_order`, `eq_with_order`, `partial_cmp_with_order`, `cmp_with_order`, etc.
///
/// ## Example Concurrent Usage
///
/// A toy concurrent program using the `ConcurrentOption` is demonstrated below:
///
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
///     for _ in 0..100 {
///         std::thread::sleep(std::time::Duration::from_millis(100));
///
///         let read = maybe.as_ref(Ordering::Acquire);
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
/// assert_eq!(maybe.as_ref(Ordering::Relaxed), Some(&7.to_string()));
/// ```
pub struct ConcurrentOption<T> {
    pub(crate) value: UnsafeCell<MaybeUninit<T>>,
    pub(crate) written: AtomicBool,
}

impl<T> ConcurrentOption<T> {
    pub(crate) unsafe fn value_ref(&self) -> &T {
        let x = &*self.value.get();
        x.assume_init_ref()
    }
}

unsafe impl<T: Send> Send for ConcurrentOption<T> {}

unsafe impl<T: Sync> Sync for ConcurrentOption<T> {}
