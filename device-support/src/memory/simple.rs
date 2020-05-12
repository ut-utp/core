//! Straightforward RAM-only [`Memory`] implementation.
//!
//! This can live in RAM on embedded devices because it only only tries to
//! provide a _part_ of the entire address space.
//!
//! TODO!
//!
//! [`Memory`]: lc3_traits::memory::Memory

use core::ops::{Index, IndexMut};

use lc3_isa::{Addr, Word};
use lc3_traits::memory::{Memory, ProgramMetadata, PageIndex, PAGE_SIZE_IN_WORDS};
// use lc3_traits::control::rpc::load::PageAccess;
use lc3_traits::control::load::Index as IndexWrapper;

/// We've got limited space, so here's what we'll do for now.
/// 256 Word (i.e. 512 byte) pages.
///
///   0: 0x0000 - 0x00FF :: backed (vectors)
///   1: 0x0100 - 0x01FF :: backed (vectors)
///   2: 0x0200 - 0x02FF :: backed (OS)
///   3: 0x0300 - 0x03FF :: backed (OS)
///   4: 0x0400 - 0x04FF :: backed (OS)
///   5: 0x0500 - 0x05FF :: backed (config)
///   6: 0x0600 - 0x06FF :: backed (???)
///   7: 0x0700 - 0x07FF :: backed (???)
// ///   8: 0x0800 - 0x08FF :: backed (???)
// ///   9: 0x0900 - 0x09FF :: backed (???)
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
// ///  61: 0x3D00 - 0x3DFF :: backed (user prog)
// ///  62: 0x3E00 - 0x3EFF :: backed (user prog)
// ///  63: 0x3F00 - 0x3FFF :: backed (user prog)
// ///  64: 0x4000 - 0x40FF :: backed (user prog)
// ///  65: 0x4100 - 0x41FF :: backed (user prog)
// ///  66: 0x4200 - 0x42FF :: backed (user prog)
/// ..........................................
/// 254: 0xFE00 - 0xFEFF :: backed (mem mapped special)
/// 255: 0xFF00 - 0xFFFF :: backed (mem mapped special)
///
/// 24 of these pages will occupy 12KiB of RAM, which we should be able to
/// handle.
///
#[repr(packed)]
pub struct PartialMemory {
    program_data: ProgramMetadata,
    pages: [[Word; Self::PAGE_SIZE]; 24],
    zero: Word,
    void: Word,
}

static __PARTIAL_MEM_SIZE_CHK: () = {
    let canary = [()];
    let size = core::mem::size_of::<PartialMemory>();

    canary[size - ((24 * 512) + 8 + 28)]
};

sa::const_assert!(core::mem::size_of::<PartialMemory>() == (24 * 512) + 8 + 28);

impl PartialMemory {
    const PAGE_SIZE: usize = 0x0100; // TODO: Use `PageAccess`?

    #[inline]
    fn addr_to_page(addr: Addr) -> Option<(usize, usize)> {
        let offset: usize = (addr as usize) % Self::PAGE_SIZE;

        match addr {
            0x0000..=0x07FF => Some(((addr as usize / Self::PAGE_SIZE), offset)),
            0x2F00..=0x3CFF => Some(((addr as usize / Self::PAGE_SIZE) - 0x2F + 8, offset)),
            0xFE00..=0xFFFF => Some(((addr as usize / Self::PAGE_SIZE) - 0xFE + 22, offset)),
            _ => None,
        }
    }
}

impl Index<Addr> for PartialMemory {
    type Output = Word;

    #[inline]
    fn index(&self, addr: Addr) -> &Self::Output {
        match PartialMemory::addr_to_page(addr) {
            Some((page, offset)) => {
                &self.pages[page][offset]
            },
            None => &self.zero,
        }
    }
}

impl IndexMut<Addr> for PartialMemory {
    #[inline]
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        match PartialMemory::addr_to_page(addr) {
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

impl Default for PartialMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialMemory {
    #[inline]
    pub const fn new() -> Self {
        Self {
            pages: [[0; PartialMemory::PAGE_SIZE]; 24],
            program_data: ProgramMetadata::empty(),
            zero: 0,
            void: 0,
        }
    }
}

sa::const_assert_eq!(PAGE_SIZE_IN_WORDS as usize, PartialMemory::PAGE_SIZE);

impl Memory for PartialMemory {
    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]) {
        if let Some((page_idx, _)) = PartialMemory::addr_to_page(IndexWrapper(page_idx).with_offset(0)) {
            self.pages[page_idx].copy_from_slice(page)
        }
    }

    fn reset(&mut self) {
        // For now, we'll do nothing!
    }

    fn get_program_metadata(&self) -> ProgramMetadata { self.program_data.clone() }
    fn set_program_metadata(&mut self, metadata: ProgramMetadata) { self.program_data = metadata }
}

// For want of placement new:
// We can't construct this in place — as in, inside the `Interpreter` struct;
// instead we have to construct it, and then _move it_ into place. This is a
// meaningful distinction for embedded things because doing the latter requires
// allocating enough stack space for *two* instances of this type: one that's
// going to be moved and one _inside_ of the Interpreter. Even though the memory
// instance that's going to be moved can have its stack space repurposed, the
// point is that this makes the maximum stack space required greater than what
// the device actually has (this is also why we can't use the
// `InterpreterBuilder` interface on embedded, I think). It's not clear to me
// why the optimizer can't const fold the stack instance of this type away
// (perhaps if we actually had a const constructor?).
//
// As a workaround, below we implement the Memory trait on _references_ to this
// type. That way, just a reference to the initial stack allocated memory
// instance can be used and passed in as the Interpreter's memory impl.

#[deny(unconditional_recursion)]
impl Index<Addr> for &'_ mut PartialMemory {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        (&**self).index(addr)
    }
}

#[deny(unconditional_recursion)]
impl IndexMut<Addr> for &'_ mut PartialMemory {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        (&mut **self).index_mut(addr)
    }
}

#[deny(unconditional_recursion)]
impl Memory for &'_ mut PartialMemory {
    fn commit_page(&mut self, page_idx: PageIndex, page: &[Word; PAGE_SIZE_IN_WORDS as usize]) {
        (&mut **self).commit_page(page_idx, page)
    }

    fn reset(&mut self) { (&mut **self).reset() }

    fn get_program_metadata(&self) -> ProgramMetadata { (&**self).get_program_metadata() }
    fn set_program_metadata(&mut self, metadata: ProgramMetadata) { (&mut **self).set_program_metadata(metadata) }
}
