use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn some() {
    let x = ConcurrentOption::some(3.to_string());
    assert_eq!(x, ConcurrentOption::some(3.to_string()));
    assert_ne!(x, ConcurrentOption::none());

    assert!(x.is_some());
    assert!(!x.is_none());
    assert_eq!(x.state(Ordering::Relaxed), State::Some);
}

#[test]
fn none() {
    let x = ConcurrentOption::<String>::none();
    assert_ne!(x, ConcurrentOption::some(3.to_string()));
    assert_eq!(x, ConcurrentOption::none());
    assert!(!x.is_some());
    assert!(x.is_none());
    assert_eq!(x.state(Ordering::Relaxed), State::None);

    let x = ConcurrentOption::default();
    assert_ne!(x, ConcurrentOption::some(3.to_string()));
    assert_eq!(x, ConcurrentOption::none());
    assert!(!x.is_some());
    assert!(x.is_none());
    assert_eq!(x.state(Ordering::Relaxed), State::None);
}
