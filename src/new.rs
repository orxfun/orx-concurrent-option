use crate::concurrent_option::ConcurrentOption;
use std::mem::MaybeUninit;

impl<T> ConcurrentOption<T> {
    pub fn some(value: T) -> Self {
        Self {
            value: MaybeUninit::new(value).into(),
            written: true.into(),
        }
    }

    pub fn none() -> Self {
        let value = MaybeUninit::uninit();
        let value = unsafe { value.assume_init() };
        Self {
            value,
            written: false.into(),
        }
    }
}
