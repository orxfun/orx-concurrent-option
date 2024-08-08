//! # orx-concurrent-option
//!
//! [![orx-concurrent-option crate](https://img.shields.io/crates/v/orx-concurrent-option.svg)](https://crates.io/crates/orx-concurrent-option)
//! [![orx-concurrent-option documentation](https://docs.rs/orx-concurrent-option/badge.svg)](https://docs.rs/orx-concurrent-option)
//!
//! A lock-free concurrent option type which enables to safely initialize and read the data concurrently.
//!
//! ## ConcurrentOption Methods In Three Groups
//!
//! ### Methods requiring `mut self` or `self`
//!
//! Since these methods guarantee a unique access to the value, these methods are identical to those of the standard `Option`.
//!
//! Some such example methods are: `unwrap`, `take`, `insert`, etc.
//!
//! ### Methods specialized for concurrent writing
//!
//! These methods require a shared `&self` reference to update the state of the option concurrently.
//!
//! Currently available methods are: `initialize_if_none` and `initialize_unchecked`.
//!
//! See the example usage below.
//!
//! ### Read methods requiring shared `&self` reference
//!
//! These methods are similar to those of the standard `Option` except that they require an additional `Ordering` argument.
//!
//! Some such example methods are: `is_some`, `is_none`, `as_ref`, `iter`, etc.
//!
//! Additional ordering is required since it is possible that the state of the option is being updated concurrently by specialized methods discussed above.
//!
//! * `Ordering::Relaxed` can be used when the state of the option is not changed concurrently.
//! * However, it is recommended to use a stronger ordering such as `Ordering::Acquire` or `Ordering::SeqCst` when it is possible that the state of the option is being updated concurrently.
//!
//! ## Example Concurrent Usage
//!
//! A toy concurrent program using the `ConcurrentOption` is demonstrated below:
//!
//! * there exist multiple readers checking the value of an optional; they will ive
//!   * None if the value is not initialized yet;
//!   * a reference to the value otherwise.
//! * there exist multiple initializers each trying to set the value of optional;
//!   * only the first one will succeed,
//!   * since this is an initialization method, succeeding calls will safely be red.
//!
//! ```rust
//! use orx_concurrent_option::*;
//! use std::sync::atomic::Ordering;
//!
//! fn reader(maybe: &ConcurrentOption<String>) {
//!     let mut is_none_at_least_once = false;
//!     let mut is_seven_at_least_once = false;
//!     for _ in 0..100 {
//!         std::thread::sleep(std::time::Duration::from_millis(100));
//!
//!         let read = maybe.as_ref(Ordering::Acquire);
//!         let is_none = read.is_none();
//!         let is_seven = read == Some(&7.to_string());
//!
//!         assert!(is_none || is_seven);
//!
//!         is_none_at_least_once |= is_none;
//!         is_seven_at_least_once |= is_seven;
//!     }
//!     assert!(is_none_at_least_once && is_seven_at_least_once);
//! }
//!
//! fn initializer(maybe: &ConcurrentOption<String>) {
//!     for _ in 0..50 {
//!         // wait for a while to simulate a delay
//!         std::thread::sleep(std::time::Duration::from_millis(100));
//!     }
//!
//!     let _ = maybe.initialize_if_none(7.to_string());
//!
//!     for _ in 0..50 {
//!         // it is safe to call `initialize_if_none` on Some variant
//!         // it will do nothing
//!         let inserted = maybe.initialize_if_none(1_000_000.to_string());
//!         assert!(!inserted);
//!     }
//! }
//!
//! let num_readers = 8;
//! let num_writers = 8;
//!
//! let maybe = ConcurrentOption::<String>::none();
//! let maybe_ref = &maybe;
//!
//! std::thread::scope(|s| {
//!     for _ in 0..num_readers {
//!         s.spawn(|| reader(maybe_ref));
//!     }
//!     for _ in 0..num_writers {
//!         s.spawn(|| initializer(maybe_ref));
//!     }
//! });
//!
//! assert_eq!(maybe.as_ref(Ordering::Relaxed), Some(&7.to_string()));
//! ```
//!
//! ## Contributing
//!
//! Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-concurrent-option/issues/new) or create a PR.
//!
//! ## License
//!
//! This library is licensed under MIT license. See LICENSE for details.

#![warn(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]

mod common_traits;
mod concurrent;
mod concurrent_option;
mod drop;
mod into_option;
mod new;
mod option;
mod raw;

pub use common_traits::iter;

pub use concurrent_option::ConcurrentOption;
pub use into_option::IntoOption;
