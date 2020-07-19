//! An extremely naïve, terrible [`Memory` trait](lc3_traits::memory::Memory)
//! implementation called [`MemoryShim`](memory::MemoryShim).

use std::convert::TryInto;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::time::Duration;

use lc3_isa::{Addr, Word, ADDR_SPACE_SIZE_IN_WORDS, MEM_MAPPED_START_ADDR};
use lc3_isa::util::MemoryDump;
use lc3_traits::memory::Memory;
use lc3_traits::control::metadata::ProgramMetadata;
use lc3_traits::control::load::{PageIndex, Index as PIdx, PageAccess, PAGE_SIZE_IN_WORDS};

use super::error::MemoryShimError;

/// Naïve [`Memory` trait](lc3_traits::memory::Memory) implementation.
///
/// Only good for hosted platforms since we just go and use 256 KiB of stack
/// space.
#[derive(Clone)]
pub struct MemoryShim {
    pub mem: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    pub current: [Word; ADDR_SPACE_SIZE_IN_WORDS],
    pub metadata: ProgramMetadata,
}

impl Default for MemoryShim {
    fn default() -> Self {
        Self {
            mem: [0u16; ADDR_SPACE_SIZE_IN_WORDS],
            current: [0u16; ADDR_SPACE_SIZE_IN_WORDS],
            metadata: ProgramMetadata::default(),
        }
    }
}

impl MemoryShim {
    pub fn new(mem: [Word; ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
        let dump = mem.into();
        let metadata = ProgramMetadata::new(Default::default(), &dump, Duration::from_secs(0));

        Self {
            mem,
            current: mem.clone(),
            metadata,
        }
    }
}

not_wasm!{
use super::file_backed::{read_from_file, write_to_file};

impl MemoryShim {
    // TODO: if we offer `ProgramMetadata::new_modified_now` on wasm go back to having
    // this just be `MemoryShim::new`.
    pub fn new_modified_now(mem: [Word; ADDR_SPACE_SIZE_IN_WORDS]) -> Self {
        let dump = mem.into();
        let metadata = ProgramMetadata::new_modified_now(Default::default(), &dump);

        Self {
            mem,
            current: mem.clone(),
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

    // Note: only writes out the persistent copy; not the temporary changes.
    pub fn to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), MemoryShimError> {
        write_to_file(path, &self.mem)?;
        self.metadata.updated_now();

        Ok(())
    }
}}

impl From<MemoryShim> for MemoryDump {
    fn from(mem: MemoryShim) -> MemoryDump {
        mem.mem.into()
    }
}

impl Index<Addr> for MemoryShim {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        &self.current[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl IndexMut<Addr> for MemoryShim {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        &mut self.current[TryInto::<usize>::try_into(addr).unwrap()]
    }
}

impl Memory for MemoryShim {
    // Note: doesn't wipe out the corresponding page from the temporary memory.
    // Must call reset for that to happen.
    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]) {
        assert!(page_idx < MEM_MAPPED_START_ADDR.page_idx());

        self.mem[PIdx(page_idx).as_index_range()].copy_from_slice(page)
    }

    fn reset(&mut self) {
        self.current = self.mem.clone();
    }

    fn get_program_metadata(&self) -> ProgramMetadata { self.metadata.clone() }

    fn set_program_metadata(&mut self, metadata: ProgramMetadata) {
        self.metadata = metadata;
    }
}
