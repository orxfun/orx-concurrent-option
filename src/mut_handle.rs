use crate::states::*;
use std::sync::atomic::AtomicU8;

pub(crate) struct MutHandle<'a> {
    state: &'a AtomicU8,
    success_state: u8,
}

impl<'a> MutHandle<'a> {
    pub fn try_get(state: &'a AtomicU8, initial_state: u8, success_state: u8) -> Option<Self> {
        match state
            .compare_exchange(initial_state, RESERVED, ORDER_LOAD, ORDER_LOAD)
            .is_ok()
        {
            true => Some(Self {
                state,
                success_state,
            }),
            false => None,
        }
    }
}

impl<'a> Drop for MutHandle<'a> {
    fn drop(&mut self) {
        self.state
            .compare_exchange(RESERVED, self.success_state, ORDER_STORE, ORDER_STORE)
            .expect("Failed to update the concurrent state after concurrent state mutation");
    }
}
