use crate::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    pub fn insert_if_none(&self, value: T) -> bool {
        const ORDER_LOAD: Ordering = Ordering::SeqCst;
        const ORDER_STORE: Ordering = Ordering::SeqCst;

        match self.written.load(ORDER_LOAD) {
            true => false,
            false => {
                let x = unsafe { self.maybe_uninit_mut() };
                x.write(value);
                self.written.store(true, ORDER_STORE);
                true
            }
        }
    }
}
