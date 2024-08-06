use std::{mem::MaybeUninit, sync::atomic::AtomicBool};

pub struct ConcurrentOption<T> {
    pub(crate) value: MaybeUninit<T>,
    pub(crate) written: AtomicBool,
}

impl<T> ConcurrentOption<T> {}
