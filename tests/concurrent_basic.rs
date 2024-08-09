use orx_concurrent_option::*;

#[test]
fn initialize_if_none() {
    let x = ConcurrentOption::<String>::none();
    let inserted = x.initialize_if_none(3.to_string());
    assert!(inserted);
    assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));

    let x = ConcurrentOption::some(7.to_string());
    let inserted = x.initialize_if_none(3.to_string());
    assert!(!inserted);
    assert_eq!(unsafe { x.as_ref() }, Some(&7.to_string()));
}

#[test]
#[cfg(not(miri))]
fn initialize_unchecked() {
    let x = ConcurrentOption::<String>::none();
    unsafe { x.initialize_unchecked(3.to_string()) };
    assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));

    let x = ConcurrentOption::some(7.to_string());
    unsafe { x.initialize_unchecked(3.to_string()) };
    assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));
}

#[test]
fn map() {
    let x = ConcurrentOption::<String>::none();
    let len = x.map(|x| x.len());
    assert_eq!(len, None);

    let x = ConcurrentOption::some(7.to_string());
    let len = x.map(|x| x.len());
    assert_eq!(len, Some(1));
}
