//! Utilities for test running.

use std::thread;
use std::time::{Instant, Duration};

const STACK_SIZE: usize = 32 * 1024 * 1024;

// TODO, low priority: this would be nice to turn into an attribute we can just
// stick on test functions:
// ```rust
// #[test]
// #[with_larger_stack]
// fn foo() { ... }
// ```
pub fn with_larger_stack<F: FnOnce() + Send + 'static>(n: Option<String>, f: F) {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .name(n.unwrap_or_else(||
            thread::current()
                .name()
                .map(String::from)
                .unwrap_or(String::from("Test Thread"))))
        .spawn(f)
        .unwrap();

    // Note: this still produces errors even when functions are marked with
    // `#[should_panic]` but it actually has nothing to do with the panic
    // machinery; it's because threads don't propogate their stdout/stderrs to
    // spawned threads. Here's a gh issue about it:
    // https://github.com/rust-lang/rust/issues/42474
    //
    // I've tried:
    //   - Using the forbidden `set_panic` and `set_print` that libtest uses:
    //       * std::io::set_panic(Some(Box::new(stderr)));
    //       * std::io::set_print(Some(Box::new(stdout)));
    //     Which inexplicably doesn't work.
    //   - Stealing the parent thread's panic handler:
    //       * std::panic::set_hook({std::panic::take_hook() (in parent)});
    //     Which understandably doesn't work since the panics are actually being
    //     propagated just fine.
    //   - Wrapping the function that runs in the child in a `catch_unwind` and
    //     then unwrapping the panic manually in the parent thread:
    //       * ```
    //         .spawn(|| catch_unwind(|| {
    //                 std::panic::set_hook(Box::new(|_| {}));
    //                 // std::io::set_panic(Some(Box::new(stderr)));
    //                 // std::io::set_print(Some(Box::new(stdout)));
    //                 println!("you shouldn't see this");
    //                 f()
    //             }))
    //             .unwrap();
    //
    //         child.join().unwrap().unwrap();
    //         ```
    //     Which also doesn't work, because again: the panicking is not the
    //     problem.
    //
    // So, I think we just have to live with it for now. Hopefully our tests
    // aren't noisy.

    child.join().unwrap();
}

// Won't work as expected for tolerances greater than half u16 width.
// ...but that wouldn't test anything anyway.
pub fn assert_is_about(actual: u16, expected: u16, tolerance: u16) {
    let min = expected.wrapping_sub(tolerance);
    let max = expected.wrapping_add(tolerance);
    let above_min = min <= actual;
    let below_max = actual <= max;
    if min <= max {
        assert!(above_min && below_max, "{:?} not between {:?} and {:?}", actual, min, max);
    } else {
        assert!(above_min || below_max, "{:?} not above {:?} or below {:?}", actual, min, max);
    }
}

pub fn run_periodically_for_a_time<R, F: FnMut(Duration) -> R>(
    period: Duration,
    duration: Duration,
    mut func: F,
) -> Vec<(Duration, R)> {
    let start = Instant::now();
    let mut record = Vec::with_capacity(
        /*duration / period*/
        (duration.as_millis() / period.as_millis()) as usize
    );

    while Instant::now().duration_since(start) <= duration {
        let next_wake_time = period * (record.len() as u32 + 1);
        let sleep_time = next_wake_time - Instant::now().duration_since(start);
        thread::sleep(sleep_time);

        let elapsed = Instant::now().duration_since(start);
        record.push((elapsed, (func)(elapsed)));
    }

    record
}

