#![doc = include_str!("../README.md")]
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
#![no_std]

mod common_traits;
mod concurrent;
mod concurrent_option;
mod drop;
mod exclusive;
mod into;
mod into_option;
mod mut_handle;
mod new;
mod option;
mod raw;
mod states;
mod with_order;

pub use common_traits::iter;

pub use concurrent_option::ConcurrentOption;
pub use into_option::IntoOption;
pub use states::State;
