use orx_concurrent_option::*;
use std::time::Duration;
use test_case::test_matrix;

#[test_matrix(
    [2, 4, 8, 16],
    [false, true]
)]
fn concurrent_take_single_writer(num_readers: usize, do_sleep: bool) {
    let maybe = ConcurrentOption::some(7.to_string());
    let maybe_ref = &maybe;

    std::thread::scope(|s| {
        for _ in 0..(num_readers / 2) {
            s.spawn(move || reader(do_sleep, maybe_ref));
        }

        s.spawn(move || taker(do_sleep, maybe_ref));

        for _ in 0..(num_readers / 2) {
            s.spawn(move || reader(do_sleep, maybe_ref));
        }
    });
}

#[test_matrix(
    [4, 8],
    [2, 4, 8, 16],
    [false, true]
)]
fn concurrent_take_multiple_writer(num_writers: usize, num_readers: usize, do_sleep: bool) {
    let maybe = ConcurrentOption::some(7.to_string());
    let maybe_ref = &maybe;

    std::thread::scope(|s| {
        for _ in 0..(num_writers / 2) {
            s.spawn(move || taker(do_sleep, maybe_ref));
        }

        for _ in 0..num_readers {
            s.spawn(move || reader(do_sleep, maybe_ref));
        }

        for _ in 0..(num_writers / 2) {
            s.spawn(move || taker(do_sleep, maybe_ref));
        }
    });
}

// helpers
fn reader(do_sleep: bool, maybe: &ConcurrentOption<String>) {
    for _ in 0..100 {
        sleep(do_sleep);
        let is_none_or_seven = maybe.map(|x| x == &7.to_string()).unwrap_or(true);
        assert!(is_none_or_seven);
    }
}

fn taker(do_sleep: bool, maybe: &ConcurrentOption<String>) {
    for i in 0..100 {
        sleep(do_sleep);
        if i == 50 {
            let _ = maybe.take();
        }
    }
}

fn sleep(do_sleep: bool) {
    if do_sleep {
        let duration = Duration::from_millis(24);
        std::thread::sleep(duration);
    }
}
