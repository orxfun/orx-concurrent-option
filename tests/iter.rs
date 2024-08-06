use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn iter_when_none() {
    fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let x = ConcurrentOption::<String>::none();
    validate(x.iter(Ordering::Relaxed));
    validate(x.iter(Ordering::Relaxed).rev());
    validate(x.into_iter());
}

#[test]
fn iter_when_some() {
    fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&3.to_string()));
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let x = ConcurrentOption::some(3.to_string());
    validate(x.iter(Ordering::Relaxed));
    validate(x.iter(Ordering::Relaxed).rev());
    validate(x.into_iter());
}
