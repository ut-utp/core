//! TODO!

pub use crate::control::metadata::ProgramMetadata;
pub use crate::control::load::{PageIndex, PAGE_SIZE_IN_WORDS};

use lc3_isa::{Addr, Word};

use core::ops::{Index, IndexMut};

// Changes written with commit_page persist when reset is called.
// Changes written with write_word must not.
pub trait Memory: Index<Addr, Output = Word> + IndexMut<Addr, Output = Word> {
    fn read_word(&self, addr: Addr) -> Word {
        self[addr]
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self[addr] = word;
    }

    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]);
    fn reset(&mut self);

    fn get_program_metadata(&self) -> ProgramMetadata;
    fn set_program_metadata(&mut self, metadata: ProgramMetadata);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MemoryStub(Word);

impl Index<Addr> for MemoryStub {
    type Output = Word;

    fn index(&self, _idx: Addr) -> &Word {
        &self.0
    }
}

impl IndexMut<Addr> for MemoryStub {
    fn index_mut(&mut self, _idx: Addr) -> &mut Word {
        &mut self.0
    }
}

impl Memory for MemoryStub {
    fn commit_page(&mut self, _page_idx: PageIndex, _page: &[Word; PAGE_SIZE_IN_WORDS as usize]) { }

    fn reset(&mut self) { self.0 = 0 }

    fn get_program_metadata(&self) -> ProgramMetadata {
        ProgramMetadata::default()
    }

    fn set_program_metadata(&mut self, _metadata: ProgramMetadata) { }
}
