use super::{Addr, Word};
use core::future::Future;

pub const MAX_BREAKPOINTS: usize = 10;
pub const MAX_MEMORY_WATCHES: usize = 10;

pub enum Event {
    Breakpoint { addr: Addr },
    MemoryWatch { addr: Addr, data: Word },
    Interrupted // If we get paused or stepped, this is returned.
}

pub enum State {
    Paused,
    RunningUntilEvent,
}

pub trait Control {
    type EventFuture: Future<Output=Event>;

    fn get_pc(&self) -> Addr;
    fn set_pc(&mut self, addr: Addr); // Should be infallible.

    fn get_register(&self, reg_num: u8) -> Word;
    fn set_register(&mut self, reg_num: u8, data: Word); // Should be infallible.

    fn get_psr(&self) -> Word;
    fn set_psr(&mut self, data: Word); // Should be infallible.

    fn get_registers_and_friends(&self) -> ([Word; 8], Word, Word) {
        let mut regs = [0; 8];
        (0..=8).for_each(|i| regs[i] = self.get_register(i as u8));

        (regs, self.get_pc(), self.get_psr())
    }

    fn write_word(&mut self, addr: Addr, word: Word);
    fn read_word(&self, addr: Addr) -> Word;
    fn flush_memory(&self) -> Result<(), ()>;

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>;
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS];
    fn get_max_breakpoints() -> usize { MAX_BREAKPOINTS }

    fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>;
    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES];
    fn get_max_memory_watches() -> usize { MAX_MEMORY_WATCHES }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.
    fn step(&mut self);
    fn pause(&mut self);

    fn get_state(&self) -> State;
}
