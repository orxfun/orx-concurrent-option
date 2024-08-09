use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn initialize_if_none() {
    let x = ConcurrentOption::<String>::none();
    let inserted = x.initialize_if_none(3.to_string());
    assert!(inserted);
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&3.to_string()));

    let x = ConcurrentOption::some(7.to_string());
    let inserted = x.initialize_if_none(3.to_string());
    assert!(!inserted);
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&7.to_string()));
}

#[test]
#[cfg(not(miri))]
fn initialize_unchecked() {
    let x = ConcurrentOption::<String>::none();
    unsafe { x.initialize_unchecked(3.to_string()) };
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&3.to_string()));

    let x = ConcurrentOption::some(7.to_string());
    unsafe { x.initialize_unchecked(3.to_string()) };
    assert_eq!(x.as_ref_with_order(Ordering::Relaxed), Some(&3.to_string()));
}

#[test]
fn map_ref() {
    let x = ConcurrentOption::<String>::none();
    let len = x.map_ref(|x| x.len());
    assert_eq!(len, None);

    let x = ConcurrentOption::some(7.to_string());
    let len = x.map_ref(|x| x.len());
    assert_eq!(len, Some(1));
}
