use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn iter_mut_when_none() {
    fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a mut String>) {
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let mut x = ConcurrentOption::<String>::none();
    validate(x.exclusive_iter_mut());
    validate(x.exclusive_iter_mut().rev());
    validate((&mut x).into_iter());
}

#[test]
fn iter_mut_when_some() {
    fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a mut String>) {
        assert_eq!(iter.len(), 1);
        *iter.next().unwrap() = 7.to_string();
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let mut x = ConcurrentOption::some(3.to_string());
    validate(x.exclusive_iter_mut());
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));

    let mut x = ConcurrentOption::some(3.to_string());
    validate(x.exclusive_iter_mut().rev());
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));

    let mut x = ConcurrentOption::some(3.to_string());
    validate((&mut x).into_iter());
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
}
