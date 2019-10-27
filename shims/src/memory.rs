//! An extremely naive, terrible [`Memory` trait](crate::memory::Memory)
//! implementation called [`MemoryShim`](memory::MemoryShim).

use lc3_isa::{Addr, Word};
use lc3_traits::memory::{Memory, MemoryMiscError};

use core::mem::size_of;

const fn pow_of_two(exp: usize) -> usize {
    1 << exp
}

const ADDR_SPACE_SIZE_IN_WORDS: usize = (pow_of_two(size_of::<Addr>() * 8) / size_of::<Word>());
// const MEMORY_SIZE_IN_WORDS: usize = ((core::u16::MAX as usize + 1) / 2);

/// Naive [`Memory` trait](crate::memory::Memory) implementation.
///
/// Only good for hosted platforms since we just go and use 128 KiB of stack
/// space.
pub struct MemoryShim {
    persistent: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    staging: [Word; ADDR_SPACE_SIZE_IN_WORDS],
}

impl Default for MemoryShim {
    fn default() -> Self {
        Self {
            persistent: [0u16; ADDR_SPACE_SIZE_IN_WORDS],
            staging: [0u16; ADDR_SPACE_SIZE_IN_WORDS],
        }
    }
}

impl MemoryShim {
    fn new(memory: [Word; ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
        Self {
            persistent: memory,
            staging: memory.clone(),
        }
    }

    fn dump_to_file() {
        unimplemented!()
        // TODO!
    }

    fn from_file() {
        unimplemented!()
        // TODO!
    }
}

impl Memory for MemoryShim {
    fn read_word(&self, addr: Addr) -> Word {
        self.staging[addr as usize]
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.staging[addr as usize] = word;
    }

    fn commit(&mut self) -> Result<(), MemoryMiscError> {
        self.persistent = self.staging.clone();

        Ok(())
    }
}
