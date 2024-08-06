use crate::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    pub fn raw_get(&self, order: Ordering) -> Option<*const T> {
        match self.written.load(order) {
            true => {
                let x = unsafe { &*self.value.get() };
                Some(x.as_ptr())
            }
            false => None,
        }
    }

    pub fn raw_get_mut(&self, order: Ordering) -> Option<*mut T> {
        match self.written.load(order) {
            true => {
                let x = unsafe { &mut *self.value.get() };
                Some(x.as_mut_ptr())
            }
            false => None,
        }
    }
}
