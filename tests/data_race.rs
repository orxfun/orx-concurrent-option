use orx_concurrent_option::*;
use std::sync::atomic::Ordering;

#[test]
fn racy_concurrent_option_replace_vs_partial_eq() {
    let (foo, bar): (ConcurrentOption<bool>, _) = (true.into(), true.into());
    std::thread::scope(|s| {
        s.spawn(|| {
            let _ = foo.replace(false);
        });
        let _ = foo == bar;
    });
}

#[test]
fn racy_concurrent_option_replace_vs_partial_cmp_with_order() {
    let (foo, bar): (ConcurrentOption<bool>, _) = (true.into(), true.into());
    std::thread::scope(|s| {
        s.spawn(|| {
            let _ = foo.replace(false);
        });
        let _ = foo.partial_cmp_with_order(&bar, Ordering::SeqCst);
    });
}

#[test]
fn racy_concurrent_option_replace_vs_clone() {
    let foo: ConcurrentOption<String> = "foo".to_string().into();
    std::thread::scope(|s| {
        s.spawn(|| {
            let _ = foo.replace("bar".to_string());
        });
        let _ = foo.clone();
    });
}

#[test]
fn racy_concurrent_option_replace_vs_clone_with_order() {
    let foo: ConcurrentOption<String> = "foo".to_string().into();
    std::thread::scope(|s| {
        s.spawn(|| {
            let _ = foo.replace("bar".to_string());
        });
        let _ = foo.clone_with_order(Ordering::SeqCst);
    });
}

#[test]
fn racy_concurrent_option_replace_vs_debug() {
    let foo: ConcurrentOption<String> = "foo".to_string().into();
    std::thread::scope(|s| {
        s.spawn(|| {
            let _ = foo.replace("bar".to_string());
        });
        let _ = format!("{:?}", foo);
    });
}

#[test]
fn racy_concurrent_option_replace_vs_into_iter() {
    let foo: ConcurrentOption<String> = "foo".to_string().into();
    std::thread::scope(|s| {
        s.spawn(|| {
            let _ = foo.replace("bar".to_string());
        });
        for x in &foo {
            let _ = x.len();
        }
    });
}
