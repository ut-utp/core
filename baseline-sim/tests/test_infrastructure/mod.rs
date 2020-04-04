//! Functions and bits that are useful for testing the interpreter.

pub use lc3_isa::{insn, Addr, Instruction, Reg, Word};
pub use lc3_shims::memory::MemoryShim;
pub use lc3_shims::peripherals::PeripheralsShim;
pub use lc3_baseline_sim::interp::PeripheralInterruptFlags;

mod runner;
#[macro_use] mod macros;
mod misc;

pub use runner::*;
pub use misc::*;
