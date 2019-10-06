use super::{Addr, Word};

pub trait Memory {
    fn read_word(&self, addr: Addr) -> Word;
    fn write_word(&mut self, addr: Addr, word: Word);

    fn flush(&self) -> Result<(), ()>;
}

pub struct MemoryShim {
    memory: [Word; ((core::u16::MAX) / 2) as usize],
}

impl Default for MemoryShim {
    fn default() -> Self {
        Self {
            memory: [0u16; ((core::u16::MAX) / 2) as usize]
        }
    }
}

impl Memory for MemoryShim {
    fn read_word(&self, addr: Addr) -> Word {
        self.memory[addr as usize]
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.memory[addr as usize] = word;
    }

    fn flush(&self) -> Result<(), ()> { Ok(()) }
}
