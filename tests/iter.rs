use core::sync::atomic::Ordering;
use orx_concurrent_option::*;

#[test]
fn iter_with_order_when_none() {
    fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let x = ConcurrentOption::<String>::none();
    validate(unsafe { x.iter() });
    validate(unsafe { x.iter_with_order(Ordering::Relaxed) }.rev());
    validate((&x).into_iter());

    fn validate_value(mut iter: impl ExactSizeIterator<Item = String>) {
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let x = ConcurrentOption::<String>::none();
    validate_value(unsafe { x.iter_with_order(Ordering::Acquire) }.cloned());
    validate_value(unsafe { x.iter() }.rev().cloned());
    validate_value(x.into_iter());
}

#[test]
fn exclusive_iter_when_some() {
    fn validate<'a>(mut iter: impl ExactSizeIterator<Item = &'a String>) {
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&3.to_string()));
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let x = ConcurrentOption::some(3.to_string());
    unsafe {
        validate(x.iter_with_order(Ordering::Relaxed));
        validate(x.iter_with_order(Ordering::Relaxed).rev());
    }
    validate((&x).into_iter());

    fn validate_value(mut iter: impl ExactSizeIterator<Item = String>) {
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(3.to_string()));
        assert_eq!(iter.len(), 0);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }

    let x = ConcurrentOption::some(3.to_string());
    unsafe {
        validate_value(x.iter_with_order(Ordering::Relaxed).cloned());
        validate_value(x.iter_with_order(Ordering::SeqCst).rev().cloned());
    }
    validate_value(x.into_iter());
}
