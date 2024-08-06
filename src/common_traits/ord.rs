use crate::ConcurrentOption;
use std::cmp::Ordering::*;

impl<T: PartialOrd> PartialOrd for ConcurrentOption<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (
            self.as_ref(std::sync::atomic::Ordering::Relaxed),
            other.as_ref(std::sync::atomic::Ordering::Relaxed),
        ) {
            (Some(l), Some(r)) => l.partial_cmp(r),
            (Some(_), None) => Some(Greater),
            (None, Some(_)) => Some(Less),
            (None, None) => Some(Equal),
        }
    }
}

impl<T: Ord> Ord for ConcurrentOption<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (
            self.as_ref(std::sync::atomic::Ordering::Relaxed),
            other.as_ref(std::sync::atomic::Ordering::Relaxed),
        ) {
            (Some(l), Some(r)) => l.cmp(r),
            (Some(_), None) => Greater,
            (None, Some(_)) => Less,
            (None, None) => Equal,
        }
    }
}
