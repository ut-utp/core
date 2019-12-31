use core::ops::{Index, IndexMut};

use lc3_isa::{Addr, Word};
use lc3_traits::memory::{Memory, MemoryMiscError};

/// We've got limited space, so here's what we'll do for now.
/// 256 Word (i.e. 512 byte) pages.
///
///   0: 0x0000 - 0x00FF :: backed (vectors)
///   1: 0x0100 - 0x01FF :: backed (vectors)
///   2: 0x0200 - 0x02FF :: backed (OS)
///   3: 0x0300 - 0x03FF :: backed (OS)
///   4: 0x0400 - 0x04FF :: backed (config)
/// ..........................................
///  47: 0x2F00 - 0x2FFF :: backed (OS stack)
///  48: 0x3000 - 0x30FF :: backed (user prog)
///  49: 0x3100 - 0x31FF :: backed (user prog)
///  50: 0x3200 - 0x32FF :: backed (user prog)
///  51: 0x3300 - 0x33FF :: backed (user prog)
///  52: 0x3400 - 0x34FF :: backed (user prog)
///  53: 0x3500 - 0x35FF :: backed (user prog)
///  54: 0x3600 - 0x36FF :: backed (user prog)
///  55: 0x3700 - 0x37FF :: backed (user prog)
///  56: 0x3800 - 0x38FF :: backed (user prog)
///  57: 0x3900 - 0x39FF :: backed (user prog)
///  58: 0x3A00 - 0x3AFF :: backed (user prog)
///  59: 0x3B00 - 0x3BFF :: backed (user prog)
///  60: 0x3C00 - 0x3CFF :: backed (user prog)
///  61: 0x3D00 - 0x3DFF :: backed (user prog)
///  62: 0x3E00 - 0x3EFF :: backed (user prog)
///  63: 0x3F00 - 0x3FFF :: backed (user prog)
/// ..........................................
/// 254: 0xFE00 - 0xFEFF :: backed (mem mapped special)
/// 255: 0xFF00 - 0xFFFF :: backed (mem mapped special)
///
/// 24 of these pages will occupy 12KiB of RAM, which we should be able to
/// handle.
///
struct Tm4cMemory {
    pages: [[Word; Self::PAGE_SIZE]; 24],
    zero: Word,
    void: Word,
}

impl Tm4cMemory {
    const PAGE_SIZE: usize = 0x0100;

    fn addr_to_page(addr: Addr) -> Option<(usize, usize)> {
        let offset: usize = (addr as usize) % Self::PAGE_SIZE;

        match addr {
            0x0000..=0x04FF => Some(((addr as usize / Self::PAGE_SIZE), offset)),
            0x2F00..=0x3FFF => Some(((addr as usize / Self::PAGE_SIZE) - 0x2F + 5, offset)),
            0xFE00..=0xFEFF => Some(((addr as usize / Self::PAGE_SIZE) - 0xFE + 22, offset)),
            _ => None,
        }
    }
}

impl Index<Addr> for Tm4cMemory {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        match Tm4cMemory::addr_to_page(addr) {
            Some((page, offset)) => {
                &self.pages[page][offset]
            },
            None => &self.zero,
        }
    }
}

impl IndexMut<Addr> for Tm4cMemory {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        match Tm4cMemory::addr_to_page(addr) {
            Some((page, offset)) => {
                &mut self.pages[page][offset]
            },
            None => {
                self.void = 0;
                &mut self.void
            },
        }
    }
}

impl Default for Tm4cMemory {
    fn default() -> Self {
        Self {
            pages: [[0; Tm4cMemory::PAGE_SIZE]; 24],
            zero: 0,
            void: 0,
        }
    }
}

impl Memory for Tm4cMemory {
    fn commit(&mut self) -> Result<(), MemoryMiscError> {
        Err(MemoryMiscError) // No persistent storage for now!
    }
}
