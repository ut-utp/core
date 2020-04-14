//! Just check that the OS assembles.

extern crate lc3_test_infrastructure as lti;

use lc3_os::*;

use lti::{assert_eq, with_larger_stack};

#[test]
fn os_size() {
    with_larger_stack(None, || assert_eq!(OS.into_iter().count(), 0x0529));
}
