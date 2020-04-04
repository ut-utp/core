//! Utilities for test running.

use std::thread;

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
