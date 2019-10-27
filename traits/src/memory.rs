use super::{Addr, Word};

pub struct MemoryMiscError;

pub trait Memory {
    fn read_word(&self, addr: Addr) -> Word;
    fn write_word(&mut self, addr: Addr, word: Word);

    fn commit(&mut self) -> Result<(), MemoryMiscError>;
}
