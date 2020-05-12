//! Tools for testing against the [lc3tools] simulator.
//!
//! [lc3tools]: https://github.com/chiragsakhuja/lc3tools

#[macro_use] mod macros;
mod runner;

pub use runner::*;

#[doc(no_inline)]
pub use crate::lc3_sequence;
