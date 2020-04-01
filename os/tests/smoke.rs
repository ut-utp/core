//! Just check that the OS assembles.

use lc3_os::*;

use pretty_assertions::assert_eq;

mod common;
use common::with_larger_stack;

#[test]
fn os_size() {
    with_larger_stack(|| assert_eq!(OS.into_iter().count(), 0x0524));
}
