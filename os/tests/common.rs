//! Utilities for test running.

use std::thread;

const STACK_SIZE: usize = 32 * 1024 * 1024;

pub fn with_larger_stack<F: FnOnce() + Send + 'static>(f: F) {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(f)
        .unwrap();

    child.join().unwrap();
}
