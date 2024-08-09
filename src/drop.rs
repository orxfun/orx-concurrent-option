use crate::{
    concurrent_option::ConcurrentOption,
    states::{RESERVED, SOME},
};
use std::sync::atomic::Ordering;

impl<T> Drop for ConcurrentOption<T> {
    fn drop(&mut self) {
        match self.state.load(Ordering::Relaxed) {
            SOME => {
                let x = unsafe { &mut *self.value.get() };
                unsafe { x.assume_init_drop() };
            }
            RESERVED => {
                panic!("ConcurrentOption is dropped while its value is being written.")
            }
            _ => {}
        }
    }
}
