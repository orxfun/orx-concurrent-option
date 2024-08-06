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

    #[cfg(feature = "experimental")]
    pub unsafe fn take_x(&self) -> Option<T> {
        const ORDER_LOAD: Ordering = Ordering::SeqCst;

        let x = self
            .written
            .compare_exchange(true, false, ORDER_LOAD, ORDER_LOAD);

        match x.is_ok() {
            true => {
                let x = self.maybe_uninit();
                Some(std::mem::MaybeUninit::assume_init_read(x))
            }
            false => None,
        }
    }
}
