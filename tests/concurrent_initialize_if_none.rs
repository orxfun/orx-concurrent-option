use orx_concurrent_option::*;
use std::{sync::atomic::Ordering, time::Duration};
use test_case::test_matrix;

#[test_matrix(
    [2, 4, 8, 16],
    [false, true],
    [Ordering::SeqCst, Ordering::Acquire]
)]
fn concurrent_initialize_if_none_single_writer(
    num_readers: usize,
    do_sleep: bool,
    read_order: Ordering,
) {
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

#[test_matrix(
    [4, 8],
    [2, 4, 8, 16],
    [false, true],
    [Ordering::SeqCst, Ordering::Acquire]
)]
fn concurrent_initialize_if_none_multiple_writer(
    num_writers: usize,
    num_readers: usize,
    do_sleep: bool,
    read_order: Ordering,
) {
    let maybe = ConcurrentOption::<String>::none();
    let maybe_ref = &maybe;

    std::thread::scope(|s| {
        for _ in 0..(num_writers / 2) {
            s.spawn(move || write_multi(do_sleep, maybe_ref));
        }

        for _ in 0..num_readers {
            s.spawn(move || read(do_sleep, maybe_ref, read_order));
        }

        for _ in 0..(num_writers / 2) {
            s.spawn(move || write_multi(do_sleep, maybe_ref));
        }
    });
}

// helpers
fn read(do_sleep: bool, maybe_ref: &ConcurrentOption<String>, read_order: Ordering) {
    for _ in 0..100 {
        sleep(do_sleep);
        let read = maybe_ref.as_ref_with_order(read_order);
        let is_none = read.is_none();
        let is_seven = read == Some(&7.to_string());
        assert!(is_none || is_seven);
    }
}

fn write_single(do_sleep: bool, maybe_ref: &ConcurrentOption<String>) {
    for i in 0..100 {
        sleep(do_sleep);
        match i {
            40 => {
                let inserted = maybe_ref.initialize_if_none(7.to_string());
                assert!(inserted);
            }
            70 => {
                let inserted = maybe_ref.initialize_if_none(111.to_string());
                assert!(!inserted);
            }
            _ => {}
        }
    }
}

fn write_multi(do_sleep: bool, maybe_ref: &ConcurrentOption<String>) {
    for i in 0..100 {
        sleep(do_sleep);
        match i {
            40 | 70 => {
                let _ = maybe_ref.initialize_if_none(7.to_string());
            }
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
