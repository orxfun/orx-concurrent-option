use orx_concurrent_option::*;

#[test]
fn concurrent_init_and_read() {
    fn reader(maybe: &ConcurrentOption<String>) {
        let mut is_none_at_least_once = false;
        let mut is_seven_at_least_once = false;
        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(100));

            let read = unsafe { maybe.as_ref() };
            let is_none = read.is_none();
            let is_seven = read == Some(&7.to_string());

            assert!(is_none || is_seven);

            is_none_at_least_once |= is_none;
            is_seven_at_least_once |= is_seven;
        }
        assert!(is_none_at_least_once && is_seven_at_least_once);
    }

    fn initializer(maybe: &ConcurrentOption<String>) {
        for _ in 0..50 {
            // wait for a while to simulate a delay
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = maybe.initialize_if_none(7.to_string());

        for _ in 0..50 {
            // it is safe to call `initialize_if_none` on Some variant
            // it will do nothing
            let inserted = maybe.initialize_if_none(1_000_000.to_string());
            assert!(!inserted);
        }
    }

    let num_readers = 8;
    let num_writers = 8;

    let maybe = ConcurrentOption::<String>::none();
    let maybe_ref = &maybe;

    std::thread::scope(|s| {
        for _ in 0..num_readers {
            s.spawn(|| reader(maybe_ref));
        }
        for _ in 0..num_writers {
            s.spawn(|| initializer(maybe_ref));
        }
    });

    assert_eq!(maybe.unwrap(), 7.to_string());
}
