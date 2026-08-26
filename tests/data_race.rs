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
