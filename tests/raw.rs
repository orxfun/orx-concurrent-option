use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn raw_get() {
    let x = ConcurrentOption::<String>::none();
    let p = x.raw_get();
    assert!(p.is_none());

    let x = ConcurrentOption::some(3.to_string());
    let p = x.raw_get();
    assert!(p.is_some());
    assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
}

#[test]
fn raw_get_with_order() {
    let x = ConcurrentOption::<String>::none();
    let p = x.raw_get_with_order(Ordering::Relaxed);
    assert!(p.is_none());

    let x = ConcurrentOption::some(3.to_string());
    let p = x.raw_get_with_order(Ordering::Relaxed);
    assert!(p.is_some());
    assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));
}

#[test]
fn raw_get_mut() {
    let x = ConcurrentOption::<String>::none();
    let p = x.raw_get_mut();
    assert!(p.is_none());

    let x = ConcurrentOption::some(3.to_string());
    let p = x.raw_get_mut();
    assert!(p.is_some());
    assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));

    let p = x.raw_get_mut().unwrap();
    assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));

    let p = x.raw_get_mut();
    let p = p.unwrap();
    let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    assert_eq!(unsafe { x.as_ref() }, Some(&7.to_string()));
}

#[test]
fn raw_get_mut_with_order() {
    let x = ConcurrentOption::<String>::none();
    let p = x.raw_get_mut_with_order(Ordering::Relaxed);
    assert!(p.is_none());

    let x = ConcurrentOption::some(3.to_string());
    let p = x.raw_get_mut_with_order(Ordering::Relaxed);
    assert!(p.is_some());
    assert_eq!(unsafe { p.unwrap().as_ref() }, Some(&3.to_string()));

    let p = x.raw_get_mut_with_order(Ordering::Relaxed).unwrap();
    assert_eq!(unsafe { p.as_ref() }, Some(&3.to_string()));

    let p = x.raw_get_mut_with_order(Ordering::Relaxed);
    let p = p.unwrap();
    let _ = unsafe { p.replace(7.to_string()) }; // only write leads to memory leak
    assert_eq!(unsafe { x.as_ref() }, Some(&7.to_string()));
}
