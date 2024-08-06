use orx_concurrent_option::*;

#[test]
fn option_into_option() {
    let x = Some(42.to_string());
    assert_eq!(x.into_option(), Some(42.to_string()));

    let x: Option<String> = None;
    assert_eq!(x.into_option(), None);
}

#[test]
fn concurrent_option_into_option() {
    let x = ConcurrentOption::some(42.to_string());
    assert_eq!(x.into_option(), Some(42.to_string()));

    let x: ConcurrentOption<String> = Default::default();
    assert_eq!(x.into_option(), None);
}
