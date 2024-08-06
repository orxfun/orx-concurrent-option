use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};

pub struct ConcurrentOption<T> {
    pub(crate) value: UnsafeCell<MaybeUninit<T>>,
    pub(crate) written: AtomicBool,
}

impl<T> ConcurrentOption<T> {
    pub(crate) unsafe fn value_ref(&self) -> &T {
        let x = &*self.value.get();
        x.assume_init_ref()
    }

    pub(crate) unsafe fn value_mut(&self) -> &mut T {
        let x = &mut *self.value.get();
        x.assume_init_mut()
    }
}

unsafe impl<T: Send> Send for ConcurrentOption<T> {}

unsafe impl<T: Sync> Sync for ConcurrentOption<T> {}
