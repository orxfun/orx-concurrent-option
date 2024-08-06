use crate::concurrent_option::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T> Drop for ConcurrentOption<T> {
    fn drop(&mut self) {
        if self.written.load(Ordering::Relaxed) {
            unsafe { self.value.assume_init_drop() };
        }
    }
}
