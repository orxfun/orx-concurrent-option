use crate::concurrent_option::ConcurrentOption;
use std::{fmt::Debug, sync::atomic::Ordering};

impl<T: Debug> Debug for ConcurrentOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.written.load(Ordering::Relaxed) {
            true => write!(f, "ConcurrentSome({:?})", unsafe { self.value_ref() }),
            false => write!(f, "ConcurrentNone"),
        }
    }
}
