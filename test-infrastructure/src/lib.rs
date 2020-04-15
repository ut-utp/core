//! Functions and bits that are useful for testing the interpreter.

pub use lc3_isa::{insn, Addr, Instruction, Reg, Word};
pub use lc3_shims::memory::MemoryShim;
pub use lc3_shims::peripherals::ShareablePeripheralsShim;
pub use lc3_baseline_sim::interp::{PeripheralInterruptFlags, Interpreter};
pub use lc3_baseline_sim::interp::{InstructionInterpreterPeripheralAccess};

mod runner;
#[macro_use] mod macros;
mod misc;

pub use runner::*;
pub use misc::*;

pub use pretty_assertions::*;
