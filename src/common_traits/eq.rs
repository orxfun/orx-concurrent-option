use crate::concurrent_option::ConcurrentOption;
use std::sync::atomic::Ordering;

impl<T: PartialEq> PartialEq for ConcurrentOption<T> {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.as_ref(Ordering::Relaxed),
            other.as_ref(Ordering::Relaxed),
        ) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y),
            _ => false,
        }
    }
}

impl<T: Eq> Eq for ConcurrentOption<T> {}
