use std::sync::atomic::Ordering;

pub(crate) const ORDER_LOAD: Ordering = Ordering::Acquire;
pub(crate) const ORDER_STORE: Ordering = Ordering::SeqCst;

pub(crate) const NONE: u8 = 0;
pub(crate) const RESERVED: u8 = 1;
pub(crate) const SOME: u8 = 2;

/// Concurrent state of the optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Optional has no value.
    None,
    /// Optional has some value.
    Some,
    /// Optional is currently reserved for a mutation.
    Reserved,
}

impl State {
    #[allow(clippy::panic, clippy::missing_panics_doc)]
    pub(crate) fn new(state: u8) -> Self {
        match state {
            NONE => Self::None,
            SOME => Self::Some,
            RESERVED => Self::Reserved,
            _ => panic!("should be either of the three valid states"),
        }
    }
}
