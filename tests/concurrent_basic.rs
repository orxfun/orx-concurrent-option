use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn insert_if_none() {
    let x = ConcurrentOption::<String>::none();
    let inserted = x.insert_if_none(3.to_string());
    assert!(inserted);
    assert_eq!(x.as_ref(Ordering::Relaxed), Some(&3.to_string()));

    let x = ConcurrentOption::some(7.to_string());
    let inserted = x.insert_if_none(3.to_string());
    assert!(!inserted);
    assert_eq!(x.as_ref(Ordering::Relaxed), Some(&7.to_string()));
}
