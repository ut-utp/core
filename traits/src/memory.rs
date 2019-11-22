use core::ops::{Index, IndexMut};

use lc3_isa::{Addr, Word};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MemoryMiscError;

pub trait Memory: Index<Addr, Output = Word> + IndexMut<Addr, Output = Word> {
    fn read_word(&self, addr: Addr) -> Word {
        self[addr]
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self[addr] = word;
    }

    fn commit(&mut self) -> Result<(), MemoryMiscError>;
}
