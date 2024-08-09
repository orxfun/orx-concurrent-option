use orx_concurrent_option::*;
use std::time::Duration;

#[test]
fn concurrent_read_and_write() {
    enum MutOperation {
        InitializeIfNone,
        UpdateIfSome,
        Replace,
        Take,
        TakeIf,
    }
    impl MutOperation {
        fn new(i: usize) -> Self {
            match i % 5 {
                0 => Self::InitializeIfNone,
                1 => Self::UpdateIfSome,
                2 => Self::Replace,
                3 => Self::Take,
                _ => Self::TakeIf,
            }
        }
    }

    let num_readers = 8;
    let num_writers = 8;

    let values = vec![ConcurrentOption::<String>::none(); 8];

    std::thread::scope(|s| {
        for _ in 0..num_readers {
            s.spawn(|| {
                for _ in 0..100 {
                    std::thread::sleep(Duration::from_millis(100));
                    let mut num_chars = 0;
                    for maybe in &values {
                        // concurrently access the value
                        num_chars += maybe.map(|x| x.len()).unwrap_or(0);
                    }
                    assert!(num_chars <= 100);
                }
            });
        }

        for _ in 0..num_writers {
            s.spawn(|| {
                for i in 0..100 {
                    std::thread::sleep(Duration::from_millis(100));
                    let e = i % values.len();

                    // concurrently update the option
                    match MutOperation::new(i) {
                        MutOperation::InitializeIfNone => {
                            values[e].initialize_if_none(e.to_string());
                        }
                        MutOperation::UpdateIfSome => {
                            values[e].update_if_some(|x| *x = format!("{}!", x));
                        }
                        MutOperation::Replace => {
                            values[e].replace(e.to_string());
                        }
                        MutOperation::Take => {
                            _ = values[e].take();
                        }
                        MutOperation::TakeIf => _ = values[e].take_if(|x| x.len() < 2),
                    }
                    let e = i % values.len();
                    _ = values[e].initialize_if_none(e.to_string());
                }
            });
        }
    })
}
