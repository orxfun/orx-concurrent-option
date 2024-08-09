use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn some() {
    let x = ConcurrentOption::some(3.to_string());
    assert_eq!(x, ConcurrentOption::some(3.to_string()));
    assert_ne!(x, ConcurrentOption::none());

    assert!(x.is_some_with_order(Ordering::Relaxed));
    assert!(!x.is_none_with_order(Ordering::Relaxed));
}

#[test]
fn none() {
    let x = ConcurrentOption::<String>::none();
    assert_ne!(x, ConcurrentOption::some(3.to_string()));
    assert_eq!(x, ConcurrentOption::none());
    assert!(!x.is_some_with_order(Ordering::Relaxed));
    assert!(x.is_none_with_order(Ordering::Relaxed));

    let x = ConcurrentOption::default();
    assert_ne!(x, ConcurrentOption::some(3.to_string()));
    assert_eq!(x, ConcurrentOption::none());
    assert!(!x.is_some_with_order(Ordering::Relaxed));
    assert!(x.is_none_with_order(Ordering::Relaxed));
}
