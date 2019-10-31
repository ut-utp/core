//! An extremely naive, terrible [`Memory` trait](lc3_traits::memory::Memory)
//! implementation called [`MemoryShim`](memory::MemoryShim).

use std::path::Path;
use std::fs::File;
use std::convert::TryInto;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use lc3_isa::{Addr, Word, ADDR_SPACE_SIZE_IN_WORDS, ADDR_SPACE_SIZE_IN_BYTES};
use lc3_traits::memory::{Memory, MemoryMiscError};

use super::error::MemoryShimError;

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
            persistent: memory,
            staging: memory,
        }
    }

    fn from_file<S: AsRef<Path>>(file: S) -> Result<Self, MemoryShimError> {
        let file = File::open(file)?;

        let length = file.metadata()?.len();
        if length != ADDR_SPACE_SIZE_IN_BYTES.try_into().unwrap() {
            return Err(MemoryShimError::IncorrectlySizedFile(length))
        }

        let mut buf: [Word; ADDR_SPACE_SIZE_IN_WORDS];
        file.read_u16_into::<LittleEndian>(&mut buf)?;

        Ok(Self::new(buf))
    }

    fn dump_to_file<S: AsRef<Path>>(&self, file: S) -> Result<(), MemoryShimError> {
        let file = File::create(file)?;

        for word in self.persistent.iter() {
            file.write_u16::<LittleEndian>(*word)?
        }

        Ok(())
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
