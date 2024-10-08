use core::sync::atomic::Ordering;
use orx_concurrent_option::*;

// &self

#[test]
fn is_some() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.is_some(), true);

    _ = x.exclusive_take();
    assert_eq!(x.is_some(), false);
}

#[test]
fn is_none() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.is_none(), false);

    _ = x.exclusive_take();
    assert_eq!(x.is_none(), true);
}

#[test]
fn as_ref() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(unsafe { x.as_ref() }, Some(&3.to_string()));

    _ = x.exclusive_take();
    assert_eq!(unsafe { x.as_ref() }, None);
}

#[test]
fn as_deref() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(unsafe { x.as_deref() }, Some("3"));

    _ = x.exclusive_take();
    assert_eq!(unsafe { x.as_deref() }, None);
}

// &self - with-order

#[test]
fn state() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(x.state(Ordering::Relaxed), State::Some);

    _ = x.exclusive_take();
    assert_eq!(x.state(Ordering::Relaxed), State::None);
}

#[test]
fn as_ref_with_order() {
    let mut x = ConcurrentOption::some(3.to_string());
    assert_eq!(
        unsafe { x.as_ref_with_order(Ordering::Relaxed) },
        Some(&3.to_string())
    );

    _ = x.exclusive_take();
    assert_eq!(unsafe { x.as_ref_with_order(Ordering::Relaxed) }, None);
}

#[test]
fn as_deref_with_order() {
    unsafe {
        let mut x = ConcurrentOption::some(3.to_string());
        assert_eq!(x.as_deref_with_order(Ordering::Relaxed), Some("3"));

        _ = x.exclusive_take();
        assert_eq!(x.as_deref_with_order(Ordering::Relaxed), None);
    }
}

// &mut self

#[test]
fn take() {
    let x = ConcurrentOption::some(3.to_string());
    let y = x.take();
    assert!(x.is_none());
    assert_eq!(y, Some(3.to_string()));

    let y = x.take();
    assert!(x.is_none());
    assert_eq!(y, None);
}

#[test]
fn exclusive_take() {
    let mut x = ConcurrentOption::some(3.to_string());
    let y = x.exclusive_take();
    assert!(x.is_none());
    assert_eq!(y, Some(3.to_string()));

    let y = x.exclusive_take();
    assert!(x.is_none());
    assert_eq!(y, None);
}

#[test]
fn take_if() {
    let x = ConcurrentOption::some(42);

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
fn exclusive_take_if() {
    let mut x = ConcurrentOption::some(42);

    let prev = x.exclusive_take_if(|v| {
        if *v == 42 {
            *v += 1;
            false
        } else {
            false
        }
    });
    assert_eq!(x, ConcurrentOption::some(43));
    assert_eq!(prev, None);

    let prev = x.exclusive_take_if(|v| *v == 43);
    assert_eq!(x, ConcurrentOption::<i32>::none());
    assert_eq!(prev, Some(43));
}

#[test]
fn exclusive_as_mut() {
    let mut x = ConcurrentOption::some("abc".to_string());
    _ = x.exclusive_as_mut().map(|x| {
        x.make_ascii_uppercase();
        x
    });
    unsafe {
        assert_eq!(x.as_deref_with_order(Ordering::Relaxed), Some("ABC"));
    }

    _ = x.exclusive_take();
    assert!(x.exclusive_as_mut().is_none());
}

#[test]
fn exclusive_as_deref_mut() {
    let mut x = ConcurrentOption::some("abc".to_string());
    _ = x.exclusive_as_deref_mut().map(|x| {
        x.make_ascii_uppercase();
        x
    });
    unsafe {
        assert_eq!(x.as_deref_with_order(Ordering::Relaxed), Some("ABC"));
    }

    _ = x.exclusive_take();
    assert!(x.exclusive_as_deref_mut().is_none());
}

#[test]
fn exclusive_replace() {
    let mut x = ConcurrentOption::some(2);
    let old = x.exclusive_replace(5);
    assert_eq!(x, ConcurrentOption::some(5));
    assert_eq!(old, Some(2));

    let mut x = ConcurrentOption::<u32>::none();
    let old = x.exclusive_replace(3);
    assert_eq!(x, ConcurrentOption::some(3));
    assert_eq!(old, None);
}

#[test]
fn exclusive_insert() {
    let mut opt = ConcurrentOption::<u32>::none();
    let val = opt.exclusive_insert(1);
    assert_eq!(*val, 1);
    assert_eq!(unsafe { opt.as_ref() }, Some(&1));
    let val = opt.exclusive_insert(2);
    assert_eq!(*val, 2);
    *val = 3;
    assert_eq!(opt.unwrap(), 3);
}

#[test]
fn exclusive_get_or_insert() {
    let mut x = ConcurrentOption::<u32>::none();

    {
        let y: &mut u32 = x.exclusive_get_or_insert(5);
        assert_eq!(y, &5);

        *y = 7;
    }

    assert_eq!(x, ConcurrentOption::some(7));
}

#[test]
fn exclusive_get_or_insert_with() {
    let mut x = ConcurrentOption::<u32>::none();

    {
        let y: &mut u32 = x.exclusive_get_or_insert_with(|| 5);
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
    let x = ConcurrentOption::some("air".to_string());
    assert_eq!(unsafe { x.unwrap_unchecked() }, "air".to_string());
}

// UNDEFINED
// #[test]
// #[should_panic]
// #[cfg(not(miri))]
// fn unwrap_unchecked_undefined_behavior() {
//     let x = ConcurrentOption::<String>::none();
//     assert_eq!(unsafe { x.unwrap_unchecked() }, "air".to_string());
// }

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
    unsafe {
        assert_eq!(x.filter(is_even), None);

        assert_eq!(ConcurrentOption::some(3).filter(is_even), None);

        assert_eq!(ConcurrentOption::some(4).filter(is_even), Some(&4));
    }
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
    assert_eq!(x.is_some_and(|x| *x > 1), true);

    let x = ConcurrentOption::some(0);
    assert_eq!(x.is_some_and(|x| *x > 1), false);

    let x: ConcurrentOption<u32> = ConcurrentOption::none();
    assert_eq!(x.is_some_and(|x| *x > 1), false);
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
fn xor() {
    let mut opt = ConcurrentOption::<i32>::none();
    let val = opt.exclusive_insert(1);
    assert_eq!(*val, 1);
    assert_eq!(unsafe { opt.as_ref() }, Some(&1));
    let val = opt.exclusive_insert(2);
    assert_eq!(*val, 2);
    *val = 3;
    assert_eq!(opt.unwrap(), 3);
}
