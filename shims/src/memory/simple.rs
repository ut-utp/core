//! An extremely naive, terrible [`Memory` trait](lc3_traits::memory::Memory)
//! implementation called [`MemoryShim`](memory::MemoryShim).

use std::path::Path;

use lc3_isa::{Addr, Word, ADDR_SPACE_SIZE_IN_WORDS};
use lc3_traits::memory::{Memory, MemoryMiscError};

use super::error::MemoryShimError;
use super::file_backed::{read_from_file, write_to_file};

/// Naive [`Memory` trait](lc3_traits::memory::Memory) implementation.
///
/// Only good for hosted platforms since we just go and use 256 KiB of stack
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
            persistent: [0u16; ADDR_SPACE_SIZE_IN_WORDS],
            staging: memory,
        }
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MemoryShimError> {
        let mut buf: [Word; ADDR_SPACE_SIZE_IN_WORDS];
        read_from_file(path, &mut buf);

        Ok(Self::new(buf))
    }

    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), MemoryShimError> {
        write_to_file(path, &self.persistent)
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
        self.persistent = self.staging;

        Ok(())
    }
}
