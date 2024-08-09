use std::sync::atomic::Ordering;

pub(crate) const ORDER_LOAD: Ordering = Ordering::Acquire;
pub(crate) const ORDER_STORE: Ordering = Ordering::SeqCst;

pub(crate) const NONE: u8 = 0;
pub(crate) const RESERVED: u8 = 1;
pub(crate) const SOME: u8 = 2;
