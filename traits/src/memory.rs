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

}
