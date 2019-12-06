use core::ops::{Index, IndexMut};
use lc3_isa::{Addr, Word};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
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
