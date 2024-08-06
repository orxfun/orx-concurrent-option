use orx_concurrent_option::*;
use std::{sync::atomic::Ordering, time::Duration};
use test_case::test_matrix;

#[test_matrix(
    [2, 4, 8, 16],
    [false, true],
    [Ordering::SeqCst, Ordering::Acquire]
)]
fn concurrent_initiate_unchecked(num_readers: usize, do_sleep: bool, read_order: Ordering) {
    let maybe = ConcurrentOption::<String>::none();
    let maybe_ref = &maybe;

    std::thread::scope(|s| {
        for _ in 0..(num_readers / 2) {
            s.spawn(move || read(do_sleep, maybe_ref, read_order));
        }

        s.spawn(move || write_single(do_sleep, maybe_ref));

        for _ in 0..(num_readers / 2) {
            s.spawn(move || read(do_sleep, maybe_ref, read_order));
        }
    });
}

// helpers
fn read(do_sleep: bool, maybe_ref: &ConcurrentOption<String>, read_order: Ordering) {
    for _ in 0..100 {
        sleep(do_sleep);
        let read = maybe_ref.as_ref(read_order);
        let is_none = read.is_none();
        let is_seven = read == Some(&7.to_string());
        assert!(is_none || is_seven);
    }
}

fn write_single(do_sleep: bool, maybe_ref: &ConcurrentOption<String>) {
    for i in 0..100 {
        sleep(do_sleep);
        match i {
            40 => unsafe { maybe_ref.initiate_unchecked(7.to_string()) },
            _ => {}
        }
    }
}

fn sleep(do_sleep: bool) {
    if do_sleep {
        let duration = Duration::from_millis(24);
        std::thread::sleep(duration);
    }
}
