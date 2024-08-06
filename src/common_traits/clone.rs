use crate::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T: Clone> Clone for ConcurrentOption<T> {
    fn clone(&self) -> Self {
        match self.written.load(Ordering::Relaxed) {
            true => {
                let value = unsafe { self.value.assume_init_ref() }.clone();
                Self::some(value)
            }
            false => Self::none(),
        }
    }
}
