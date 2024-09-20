use crate::states::*;
use core::sync::atomic::{AtomicU8, Ordering};

pub(crate) struct Handle<'a> {
    state: &'a AtomicU8,
    success_state: StateU8,
}

impl<'a> Handle<'a> {
    pub fn get(
        state: &'a AtomicU8,
        initial_state: StateU8,
        success_state: StateU8,
    ) -> Option<Self> {
        match state
            .compare_exchange(
                initial_state,
                RESERVED,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            true => Some(Self {
                state,
                success_state,
            }),
            false => None,
        }
    }

    pub fn spin_get(
        state: &'a AtomicU8,
        initial_state: StateU8,
        success_state: StateU8,
    ) -> Option<Self> {
        loop {
            match state.compare_exchange(
                initial_state,
                RESERVED,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(Self {
                        state,
                        success_state,
                    })
                }
                Err(previous_state) => match previous_state {
                    RESERVED => continue,
                    _ => return None,
                },
            }
        }
    }
}

impl<'a> Drop for Handle<'a> {
    fn drop(&mut self) {
        self.state
            .compare_exchange(
                RESERVED,
                self.success_state,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .expect("Failed to update the concurrent state after concurrent state mutation");
    }
}
