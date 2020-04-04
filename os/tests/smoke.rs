//! Just check that the OS assembles.

use lc3_os::*;

use pretty_assertions::assert_eq;

// Note: same as the trap tests; this is brittle and kind of bad; at some point
// it'd be good to spin off the shared testing infrastructure crate that lives
// in this workspace (TODO, low priority).
#[path = "../../baseline-sim/tests/test_infrastructure/mod.rs"]
mod common;
use common::with_larger_stack;

#[test]
fn os_size() {
    with_larger_stack(None, || assert_eq!(OS.into_iter().count(), 0x0524));
}
