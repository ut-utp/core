use super::{Addr, Word};

pub struct MemoryMiscError;

pub trait Memory {
    fn read_word(&self, addr: Addr) -> Word;
    fn write_word(&mut self, addr: Addr, word: Word);

    fn commit(&mut self) -> Result<(), MemoryMiscError>;
}

/// An extremely naive, terrible [`Memory`](trait.Memory.html) implementation.
pub struct MemoryShim {
    persistent: [Word; ((core::u16::MAX) / 2) as usize],
    staging: [Word; ((core::u16::MAX) / 2) as usize],
}

impl Default for MemoryShim {
    fn default() -> Self {
        Self {
            persistent: [0u16; ((core::u16::MAX) / 2) as usize],
            staging: [0u16; ((core::u16::MAX) / 2) as usize]
        }
    }
}

// TODO: move all shims to a module called shims
// TODO: feature gate the shims
// TODO: split gpio, adc, etc. into separate files in a peripherals module

impl MemoryShim {
    fn new(memory: [Word; ((core::u16::MAX) / 2) as usize]) -> Self {
        Self {
            persistent: memory,
            staging: memory.clone()
        }
    }

    fn dump_to_file() {
        unimplemented!()
        // TODO!
    }

    fn from_file() {
        unimplemented!()
        // TODO!
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
        self.persistent = self.staging.clone();

        Ok(())
    }
}
