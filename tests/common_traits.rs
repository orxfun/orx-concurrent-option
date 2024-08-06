use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn clone() {
    let x = ConcurrentOption::some(3.to_string());
    let y = x.clone();

    assert!(y.is_some(Ordering::Relaxed));
    assert_eq!(y.as_ref(Ordering::Relaxed), Some(&3.to_string()));
    assert_eq!(x, y);

    let x = ConcurrentOption::<String>::none();
    let y = x.clone();

    assert!(y.is_none(Ordering::Relaxed));
    assert_eq!(y.as_ref(Ordering::Relaxed), None);
    assert_eq!(x, y);
}

#[test]
fn debug() {
    let x = ConcurrentOption::some(3.to_string());
    let y = format!("{:?}", x);
    assert_eq!(y, "ConcurrentSome(\"3\")");

    let x = ConcurrentOption::<String>::none();
    let y = format!("{:?}", x);
    assert_eq!(y, "ConcurrentNone");
}

#[test]
fn partial_ord() {
    use std::cmp::Ordering::*;

    let x = ConcurrentOption::some(3);
    let y = ConcurrentOption::some(7);
    let z = ConcurrentOption::<i32>::none();

    assert_eq!(x.partial_cmp(&x), Some(Equal));
    assert_eq!(x.partial_cmp(&y), Some(Less));
    assert_eq!(x.partial_cmp(&z), Some(Greater));

    assert_eq!(y.partial_cmp(&x), Some(Greater));
    assert_eq!(y.partial_cmp(&y), Some(Equal));
    assert_eq!(y.partial_cmp(&z), Some(Greater));

    assert_eq!(z.partial_cmp(&x), Some(Less));
    assert_eq!(z.partial_cmp(&y), Some(Less));
    assert_eq!(z.partial_cmp(&z), Some(Equal));
}

#[test]
fn ord() {
    use std::cmp::Ordering::*;

    let x = ConcurrentOption::some(3);
    let y = ConcurrentOption::some(7);
    let z = ConcurrentOption::<i32>::none();

    assert_eq!(x.cmp(&x), Equal);
    assert_eq!(x.cmp(&y), Less);
    assert_eq!(x.cmp(&z), Greater);

    assert_eq!(y.cmp(&x), Greater);
    assert_eq!(y.cmp(&y), Equal);
    assert_eq!(y.cmp(&z), Greater);

    assert_eq!(z.cmp(&x), Less);
    assert_eq!(z.cmp(&y), Less);
    assert_eq!(z.cmp(&z), Equal);
}

#[test]
fn eq() {
    let x = ConcurrentOption::some(3);
    let y = ConcurrentOption::some(7);
    let z = ConcurrentOption::<i32>::none();

    assert!(x.eq(&x));
    assert!(!x.eq(&y));
    assert!(!x.eq(&z));

    assert!(!z.eq(&x));
    assert!(!z.eq(&y));
    assert!(z.eq(&z));
}

#[test]
fn from() {
    let x: ConcurrentOption<String> = 3.to_string().into();
    assert_eq!(x.as_ref(Ordering::Relaxed), Some(&3.to_string()));

    let x: ConcurrentOption<String> = Some(3.to_string()).into();
    assert_eq!(x.as_ref(Ordering::Relaxed), Some(&3.to_string()));

    let x: ConcurrentOption<String> = None.into();
    assert_eq!(x.as_ref(Ordering::Relaxed), None);
}

#[test]
fn into() {
    let x = ConcurrentOption::some(3.to_string());
    let y: Option<_> = x.into();
    assert_eq!(y, Some(3.to_string()));

    let x = ConcurrentOption::<String>::none();
    let y: Option<String> = x.into();
    assert_eq!(y, None);
}
