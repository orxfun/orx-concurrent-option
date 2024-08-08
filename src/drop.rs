use crate::concurrent_option::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T> Drop for ConcurrentOption<T> {
    fn drop(&mut self) {
        if self.written.load(Ordering::Relaxed) {
            let x = unsafe { &mut *self.value.get() };
            unsafe { x.assume_init_drop() };
        }
    }
}
