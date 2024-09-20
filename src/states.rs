use core::sync::atomic::Ordering;

/// State represented as u8.
pub type StateU8 = u8;

pub const ORDER_LOAD: Ordering = Ordering::Acquire;
pub const ORDER_STORE: Ordering = Ordering::SeqCst;

/// State where the optional does not have a value.
pub const NONE: StateU8 = 0;
/// State where the optional's value is being transitioned.
pub const RESERVED: StateU8 = 1;
/// State where the optional contains a value.
pub const SOME: StateU8 = 2;

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
    pub(crate) fn new(state: StateU8) -> Self {
        match state {
            NONE => Self::None,
            SOME => Self::Some,
            RESERVED => Self::Reserved,
            _ => panic!("should be either of the three valid states"),
        }
    }
}
