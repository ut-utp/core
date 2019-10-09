//! The [`Control` trait](trait.Control.html) and friends.
//!
//! Unlike the [`Peripherals` trait](../peripherals/trait.Peripherals.html) and
//! the [`Memory` trait](../memory/trait.Memory.html), there is no shim
//! implementation of Control; instead the 'shim' is an instruction level
//! simulator that lives in the [interp module](../interp).

use super::{Addr, Word};
use super::error::Error;
use core::future::Future;

pub const MAX_BREAKPOINTS: usize = 10;
pub const MAX_MEMORY_WATCHES: usize = 10;

pub enum Event {
    Breakpoint { addr: Addr },
    MemoryWatch { addr: Addr, data: Word },
    Interrupted, // If we get paused or stepped, this is returned.
}

pub enum State {
    Paused,
    RunningUntilEvent,
}

// TODO: derive macro to give us:
//   - an iterator through all the variants
//   - a const function with the number of variants (`Reg::num_variants()`)
pub enum Reg {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PSR,
}

pub trait Control {
    type EventFuture: Future<Output = Event>;

    fn get_pc(&self) -> Addr;
    fn set_pc(&mut self, addr: Addr); // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word;
    fn set_register(&mut self, reg: Reg, data: Word); // Should be infallible.

    fn get_registers_and_pc(&self) -> ([Word; 9], Word) {
        let mut regs = [0; 9];

        use Reg::*;
        [R0, R1, R2, R3, R4, R5, R6, R7, PSR]
            .iter()
            .enumerate()
            .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

        (regs, self.get_pc())
    }

    fn write_word(&mut self, addr: Addr, word: Word);
    fn read_word(&self, addr: Addr) -> Word;
    fn commit_memory(&self) -> Result<(), ()>;

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>;
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS];
    fn get_max_breakpoints() -> usize {
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>;
    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES];
    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.
    fn step(&mut self);
    fn pause(&mut self);

    fn get_state(&self) -> State;
}
