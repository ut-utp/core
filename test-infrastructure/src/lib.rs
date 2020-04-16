//! Functions and bits that are useful for testing the interpreter.

#[doc(no_inline)]
pub use {
    lc3_isa::{insn, Addr, Instruction, Reg, Word},
    lc3_shims::memory::MemoryShim,
    lc3_shims::peripherals::{PeripheralsShim, ShareablePeripheralsShim},
    lc3_baseline_sim::interp::{PeripheralInterruptFlags, Interpreter},
    lc3_baseline_sim::interp::{InstructionInterpreterPeripheralAccess},
};

#[doc(no_inline)]
pub use pretty_assertions::*;

mod runner;
#[macro_use] pub mod macros;
mod misc;

pub use runner::*;
pub use misc::*;
