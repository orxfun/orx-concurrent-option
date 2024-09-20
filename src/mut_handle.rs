use crate::{states::*, ConcurrentOption};
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

/// Provides a mut-handle on the optional.
pub struct MutHandle<'a, T> {
    state: &'a AtomicU8,
    success_state: StateU8,
    /// Provides direct access to the cell holding the data of the optional.
    pub value: &'a UnsafeCell<MaybeUninit<T>>,
}

impl<'a, T> MutHandle<'a, T> {
    pub(crate) fn spin_get(
        option: &'a ConcurrentOption<T>,
        initial_state: StateU8,
        success_state: StateU8,
    ) -> Option<Self> {
        loop {
            match option.state.compare_exchange(
                initial_state,
                RESERVED,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(Self {
                        state: &option.state,
                        success_state,
                        value: &option.value,
                    });
                }
                Err(previous_state) => match previous_state {
                    RESERVED => continue,
                    _ => return None,
                },
            }
        }
    }

    /// Creates a `&mut T` reference to the underlying value of the optional.
    ///
    /// # Safety
    ///
    /// This operation might lead to undefined behavior:
    /// * if we use it while other threads are accessing the data, or
    /// * if the optional `is_none` when we access the value.
    pub unsafe fn get_mut(&self) -> &mut T {
        let x = unsafe { &mut *self.value.get() };
        unsafe { MaybeUninit::assume_init_mut(x) }
    }
}

impl<'a, T> Drop for MutHandle<'a, T> {
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
