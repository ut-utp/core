//! An extremely naïve, terrible [`Memory` trait](lc3_traits::memory::Memory)
//! implementation called [`MemoryShim`](memory::MemoryShim).

use std::convert::TryInto;
use std::ops::{Index, IndexMut};
use std::path::Path;

use lc3_isa::{Addr, Word, ADDR_SPACE_SIZE_IN_WORDS};
use lc3_isa::util::MemoryDump;
use lc3_traits::memory::Memory;
use lc3_traits::control::metadata::ProgramMetadata;

use super::error::MemoryShimError;
use super::file_backed::{read_from_file, write_to_file};

/// Naïve [`Memory` trait](lc3_traits::memory::Memory) implementation.
///
/// Only good for hosted platforms since we just go and use 256 KiB of stack
/// space.
pub struct MemoryShim {
    mem: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    metadata: ProgramMetadata,
}

impl Default for MemoryShim {
    fn default() -> Self {
        Self {
            mem: [0u16; ADDR_SPACE_SIZE_IN_WORDS],
            metadata: ProgramMetadata::default(),
        }
    }
}

impl MemoryShim {
    pub fn new(mem: [Word; ADDR_SPACE_SIZE_IN_WORDS]) -> Self {

        let dump = mem.into();
        let metadata = ProgramMetadata::new_modified_now(&dump);

        Self {
            mem,
            metadata,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MemoryShimError> {
        let mut buf: [Word; ADDR_SPACE_SIZE_IN_WORDS] = [0u16; ADDR_SPACE_SIZE_IN_WORDS];
        let modified_time = read_from_file(path, &mut buf)?;

        let mut mem = Self::new(buf);
        mem.metadata.modified_on(modified_time);

        Ok(mem)
    }

    pub fn to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), MemoryShimError> {
        write_to_file(path, &self.mem)?;
        self.metadata.updated_now();

        Ok(())
    }
}

impl From<MemoryShim> for MemoryDump {
    fn from(mem: MemoryShim) -> MemoryDump {
        mem.mem.into()
    }
}

impl Index<Addr> for MemoryShim {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        &self.mem[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl IndexMut<Addr> for MemoryShim {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        &mut self.mem[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl Memory for MemoryShim {
    fn get_program_metadata(&self) -> ProgramMetadata { self.metadata.clone() }

    fn set_program_metadata(&mut self, metadata: ProgramMetadata) {
        self.metadata = metadata;
    }
}
