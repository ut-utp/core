//! The [`Control` trait](crate::control::Control) and friends.
//!
//! Unlike the [`Peripherals` trait](crate::peripherals::Peripherals) and the
//! [`Memory` trait](crate::memory::Memory), there is no shim implementation of
//! Control; instead the 'shim' is an instruction level simulator that lives in
//! the [interp module](crate::interp).

use super::error::Error;
use core::future::Future;
use lc3_isa::{Addr, Word};
use serde::{Deserialize, Serialize};

pub const MAX_BREAKPOINTS: usize = 10;
pub const MAX_MEMORY_WATCHES: usize = 10;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Event {
    Breakpoint { addr: Addr },
    MemoryWatch { addr: Addr, data: Word },
    Interrupted, // If we get paused or stepped, this is returned.
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum State {
    Paused,
    RunningUntilEvent,
}

// TODO: derive macro to give us:
//   - an iterator through all the variants
//   - a const function with the number of variants (`Reg::num_variants()`)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error>;

    // I/O Access:
    // TODO!! Does the state/reading separation make sense?
    // fn get_gpio_states();
    // fn get_gpio_reading();
    // fn get_adc_states();
    // fn get_adc_reading();
    // fn get_timer_states();
    // fn get_timer_config();
    // fn get_pwm_states();
    // fn get_pwm_config();
    // fn get_clock();

    // So with some of these functions that are basically straight wrappers over their Memory/Peripheral trait counterparts,
    // we have a bit of a choice. We can make Control a super trait of those traits so that we can have default impls of said
    // functions or we can make the implementor of Control manually wrap those functions.
    //
    // The downside to the super trait way is that it's a little weird; it requires that one massive type hold all the state
    // for all the Peripherals and Memory (and whatever the impl for Control ends up needing). You can of course store the
    // state for those things in their own types within your big type, but then to impl, say, Memory, you'd have to manually
    // pass all the calls along meaning we're back where we started.
    //
    // Associated types really don't seem to save us here (still gotta know where the state is stored which we don't know
    // when writing a default impl) and I can't think of a way that's appreciably better so I think we just have to eat it.
    //
    // We kind of got around this with the `PeripheralSet` struct in the peripherals module, but I'm not sure it'd work here.
}
