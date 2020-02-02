//! TODO!

use crate::control::metadata::ProgramMetadata;

use lc3_isa::{Addr, Word};

use core::ops::{Index, IndexMut};

pub trait Memory: Index<Addr, Output = Word> + IndexMut<Addr, Output = Word> {
    fn read_word(&self, addr: Addr) -> Word {
        self[addr]
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self[addr] = word;
    }

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
    fn get_program_metadata(&self) -> ProgramMetadata {
        ProgramMetadata::default()
    }

    fn set_program_metadata(&mut self, _metadata: ProgramMetadata) { }
}
