use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

// &self

#[test]
fn as_ref() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.as_ref(Ordering::Relaxed), Some(&3.to_string()));

    _ = x.take();
    assert_eq!(x.as_ref(Ordering::Relaxed), None);
}

#[test]
fn as_deref() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.as_deref(Ordering::Relaxed), Some("3"));

    _ = x.take();
    assert_eq!(x.as_deref(Ordering::Relaxed), None);
}

// &mut self

#[test]
fn take() {
    let mut x = ConcurrentOption::some(3.to_string());
    let y = x.take();
    assert!(x.is_none(Ordering::Relaxed));
    assert_eq!(y, Some(3.to_string()));

    let y = x.take();
    assert!(x.is_none(Ordering::Relaxed));
    assert_eq!(y, None);
}

#[test]
fn take_if() {
    let mut x = ConcurrentOption::some(42);

    let prev = x.take_if(|v| {
        if *v == 42 {
            *v += 1;
            false
        } else {
            false
        }
    });
    assert_eq!(x, ConcurrentOption::some(43));
    assert_eq!(prev, None);

    let prev = x.take_if(|v| *v == 43);
    assert_eq!(x, ConcurrentOption::<i32>::none());
    assert_eq!(prev, Some(43));
}

#[test]
fn as_mut() {
    let mut x = ConcurrentOption::some("abc".to_string());
    _ = x.as_mut().map(|x| {
        x.make_ascii_uppercase();
        x
    });
    assert_eq!(x.as_deref(Ordering::Relaxed), Some("ABC"));

    _ = x.take();
    assert!(x.as_mut().is_none());
}

#[test]
fn as_deref_mut() {
    let mut x = ConcurrentOption::some("abc".to_string());
    _ = x.as_deref_mut().map(|x| {
        x.make_ascii_uppercase();
        x
    });
    assert_eq!(x.as_deref(Ordering::Relaxed), Some("ABC"));

    _ = x.take();
    assert!(x.as_deref_mut().is_none());
}

#[test]
fn replace() {
    let mut x = ConcurrentOption::some(2);
    let old = x.replace(5);
    assert_eq!(x, ConcurrentOption::some(5));
    assert_eq!(old, Some(2));

    let mut x = ConcurrentOption::<u32>::none();
    let old = x.replace(3);
    assert_eq!(x, ConcurrentOption::some(3));
    assert_eq!(old, None);
}

#[test]
fn insert() {
    let mut opt = ConcurrentOption::<u32>::none();
    let val = opt.insert(1);
    assert_eq!(*val, 1);
    assert_eq!(opt.as_ref(Ordering::Relaxed), Some(&1));
    let val = opt.insert(2);
    assert_eq!(*val, 2);
    *val = 3;
    assert_eq!(opt.unwrap(), 3);
}

#[test]
fn get_or_insert() {
    let mut x = ConcurrentOption::<u32>::none();

    {
        let y: &mut u32 = x.get_or_insert(5);
        assert_eq!(y, &5);

        *y = 7;
    }

    assert_eq!(x, ConcurrentOption::some(7));
}

#[test]
fn get_or_insert_with() {
    let mut x = ConcurrentOption::<u32>::none();

    {
        let y: &mut u32 = x.get_or_insert_with(|| 5);
        assert_eq!(y, &5);

        *y = 7;
    }

    assert_eq!(x, ConcurrentOption::some(7));
}

// self

#[test]
fn unwrap() {
    let x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.unwrap(), 3.to_string());
}

#[test]
fn unwrap_or() {
    assert_eq!(ConcurrentOption::some("car").unwrap_or("bike"), "car");
    assert_eq!(ConcurrentOption::none().unwrap_or("bike"), "bike");
}

#[test]
fn unwrap_or_default() {
    let x = ConcurrentOption::<i32>::none();
    let y = ConcurrentOption::some(12);

    assert_eq!(x.unwrap_or_default(), 0);
    assert_eq!(y.unwrap_or_default(), 12);
}

#[test]
fn unwrap_or_else() {
    let k = 10;
    assert_eq!(ConcurrentOption::some(4).unwrap_or_else(|| 2 * k), 4);
    assert_eq!(ConcurrentOption::<i32>::none().unwrap_or_else(|| 2 * k), 20);
}

#[test]
fn unwrap_unchecked() {
    let x = ConcurrentOption::some("air");
    assert_eq!(unsafe { x.unwrap_unchecked() }, "air");
}

#[test]
#[should_panic]
#[cfg(not(miri))]
fn unwrap_unchecked_undefined_behavior() {
    let x = ConcurrentOption::<&str>::none();
    assert_eq!(unsafe { x.unwrap_unchecked() }, "air");
}

#[test]
#[should_panic]
fn unwrap_when_none() {
    let x: ConcurrentOption<String> = Default::default();
    let _ = x.unwrap();
}

#[test]
fn expect() {
    let x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.expect("ooo"), 3.to_string());
}

#[test]
#[should_panic]
fn expect_when_none() {
    let x: ConcurrentOption<String> = Default::default();
    let _ = x.expect("ooo");
}

#[test]
fn and() {
    let x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.and(Some(42)), Some(42));

    let x = ConcurrentOption::<String>::none();
    assert_eq!(x.and(Some(42)), None);
}

#[test]
fn and_then() {
    let x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.and_then(|x| x.chars().next()), Some('3'));

    let x = ConcurrentOption::<String>::none();
    assert_eq!(x.and_then(|x| x.chars().next()), None);
}

#[test]
fn cloned() {
    let x = 12;
    let opt_x = ConcurrentOption::some(&x);
    assert_eq!(opt_x, ConcurrentOption::some(&12));
    let cloned = opt_x.cloned();
    assert_eq!(cloned, Some(12));
}

#[test]
fn copied() {
    let x = 12;
    let opt_x = ConcurrentOption::some(&x);
    assert_eq!(opt_x, ConcurrentOption::some(&12));
    let copied = opt_x.copied();
    assert_eq!(copied, Some(12));
}

#[test]
fn filter() {
    fn is_even(n: &i32) -> bool {
        n % 2 == 0
    }

    let x = ConcurrentOption::<_>::none();
    assert_eq!(x.filter(is_even), None);

    assert_eq!(ConcurrentOption::some(3).filter(is_even), None);

    assert_eq!(ConcurrentOption::some(4).filter(is_even), Some(4));
}

#[test]
fn flatten() {
    let x = ConcurrentOption::some(ConcurrentOption::some(6));
    assert_eq!(Some(6), x.flatten());

    let x: ConcurrentOption<ConcurrentOption<u32>> =
        ConcurrentOption::some(ConcurrentOption::none());
    assert_eq!(None, x.flatten());

    let x: ConcurrentOption<ConcurrentOption<u32>> = ConcurrentOption::none();
    assert_eq!(None, x.flatten());

    let x = ConcurrentOption::some(Some(6));
    assert_eq!(Some(6), x.flatten());

    let x: ConcurrentOption<Option<u32>> = ConcurrentOption::some(None);
    assert_eq!(None, x.flatten());

    let x: ConcurrentOption<Option<u32>> = ConcurrentOption::none();
    assert_eq!(None, x.flatten());
}

#[test]
fn is_some_and() {
    let x = ConcurrentOption::some(2);
    assert_eq!(x.is_some_and(|x| x > 1), true);

    let x = ConcurrentOption::some(0);
    assert_eq!(x.is_some_and(|x| x > 1), false);

    let x: ConcurrentOption<u32> = ConcurrentOption::none();
    assert_eq!(x.is_some_and(|x| x > 1), false);
}

#[test]
fn map() {
    let x = ConcurrentOption::<String>::none();
    let y = x.map(|a| format!("{}!", a));
    assert!(y.is_none());

    let x = ConcurrentOption::some(3.to_string());
    let y = x.map(|a| format!("{}!", a));
    assert_eq!(y, Some(String::from("3!")));
}

#[test]
fn map_or() {
    let x = ConcurrentOption::some("foo");
    assert_eq!(x.map_or(42, |v| v.len()), 3);

    let x = ConcurrentOption::<&str>::none();
    assert_eq!(x.map_or(42, |v| v.len()), 42);
}

#[test]
fn map_or_else() {
    let k = 21;

    let x = ConcurrentOption::some("foo");
    assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 3);

    let x = ConcurrentOption::<&str>::none();
    assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 42);
}

#[test]
fn ok_or() {
    let x = ConcurrentOption::some("foo");
    assert_eq!(x.ok_or(0), Ok("foo"));

    let x = ConcurrentOption::<&str>::none();
    assert_eq!(x.ok_or(0), Err(0));
}

#[test]
fn ok_or_else() {
    let x = ConcurrentOption::some("foo");
    assert_eq!(x.ok_or_else(|| 0), Ok("foo"));

    let x = ConcurrentOption::<&str>::none();
    assert_eq!(x.ok_or_else(|| 0), Err(0));
}

#[test]
fn or() {
    let x = ConcurrentOption::some(2);
    let y = ConcurrentOption::<u32>::none();
    assert_eq!(x.or(y), Some(2));

    let x = ConcurrentOption::<u32>::none();
    let y = ConcurrentOption::some(100);
    assert_eq!(x.or(y), Some(100));

    let x = ConcurrentOption::some(2);
    let y = ConcurrentOption::some(100);
    assert_eq!(x.or(y), Some(2));

    let x: Option<u32> = None;
    let y = None;
    assert_eq!(x.or(y), None);
}

#[test]
fn or_else() {
    fn nobody() -> Option<&'static str> {
        None
    }
    fn vikings() -> ConcurrentOption<&'static str> {
        ConcurrentOption::some("vikings")
    }

    assert_eq!(
        ConcurrentOption::some("barbarians").or_else(vikings),
        Some("barbarians")
    );
    assert_eq!(
        ConcurrentOption::<&str>::none().or_else(vikings),
        Some("vikings")
    );
    assert_eq!(ConcurrentOption::<&str>::none().or_else(nobody), None);
}

#[test]
fn xor() {
    let mut opt = ConcurrentOption::<i32>::none();
    let val = opt.insert(1);
    assert_eq!(*val, 1);
    assert_eq!(opt.as_ref(Ordering::Relaxed), Some(&1));
    let val = opt.insert(2);
    assert_eq!(*val, 2);
    *val = 3;
    assert_eq!(opt.unwrap(), 3);
}

#[test]
fn zip() {
    let x = ConcurrentOption::some(1);
    let y = Some("hi");
    assert_eq!(x.zip(y), Some((1, "hi")));

    let x = ConcurrentOption::some(1);
    let z = ConcurrentOption::<u8>::none();
    assert_eq!(x.zip(z), None);
}

#[test]
fn unzip() {
    let x = ConcurrentOption::some((1, "hi"));
    let y = ConcurrentOption::<(u8, u32)>::none();

    assert_eq!(x.unzip(), (Some(1), Some("hi")));
    assert_eq!(y.unzip(), (None, None));
}
