//! Constants. (TODO)

use core::ops::Deref;
use lc3_isa::{Addr, Bits, Word, WORD_MAX_VAL, MCR as MCR_ADDRESS, PSR as PSR_ADDRESS};

use crate::interp::{InstructionInterpreter, WriteAttempt};

pub trait MemMapped: Deref<Target = Word> + Sized {
    const ADDR: Addr;

    fn with_value(value: Word) -> Self;

    fn from<I: InstructionInterpreter>(interp: &I) -> Result<Self, ()> {
        // Checked access by default:
        Ok(Self::with_value(interp.get_word(Self::ADDR)?))
    }

    fn set<I: InstructionInterpreter>(interp: &mut I, value: Word) -> WriteAttempt {
        // Checked access by default:
        interp.set_word(Self::ADDR, value)
    }

    fn update<I: InstructionInterpreter>(interp: &mut I, func: impl FnOnce(Self) -> Word) -> WriteAttempt {
        Self::set(interp, func(Self::from(interp)?))
    }

    #[doc(hidden)]
    fn write_current_value<I: InstructionInterpreter>(&self, interp: &mut I) -> WriteAttempt {
        Self::set(interp, **self)
    }
}

// struct KBSR(Word);

// impl Deref for KBSR {
//     type Target = Word;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl MemMapped for KBSR {
//     const ADDR: Addr = 0xFE00;

//     fn with_value(value: Word) -> Self {
//         Self(value)
//     }
// }

macro_rules! mem_mapped {
    ($name:ident, $address:expr, $comment:literal) => {
        #[doc=$comment]
        mem_mapped!(-; $name, $address, $comment);
    };

    (special: $name:ident, $address:expr, $comment:literal) => {
        #[doc=$comment]
        #[doc="\nDoes not produce access control violations (ACVs) when accessed!"]
        mem_mapped!(+; $name, $address, $comment);
    };

    ($special:tt; $name:ident, $address:expr, $comment:literal) => {
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $name(Word);

        impl Deref for $name {
            type Target = Word;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl MemMapped for $name {
            const ADDR: Addr = $address;

            fn with_value(value: Word) -> Self {
                Self(value)
            }

            _mem_mapped_special_access!($special);
        }
    }
}

macro_rules! _mem_mapped_special_access {
    (+) => {
        fn from<I: InstructionInterpreter>(interp: &I) -> Result<Self, ()> {
            // Special unchecked access!
            Ok(Self::with_value(interp.get_word_unchecked(Self::ADDR)))
        }

        fn set<I: InstructionInterpreter>(interp: &mut I, value: Word) -> WriteAttempt {
            // Special unchecked access!
            interp.set_word_unchecked(Self::ADDR, value);
            WriteAttempt::Success
        }
    };
    (-) => {};
}

// struct KBSR(Word);

// impl Deref for KBSR {
//     type Target = Word;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl MemMapped for KBSR {
//     const ADDR: Addr = 0xFE00;

//     fn with_value(value: Word) -> Self {
//         Self(value)
//     }
// }

mem_mapped!(KBSR, 0xFE00, "Keyboard Status Register.");
mem_mapped!(KBDR, 0xFE02, "Keyboard Data Register.");

impl KBSR {
    pub fn
}

mem_mapped!(DSR, 0xFE04, "Display Status Register.");
mem_mapped!(DDR, 0xFE06, "Display Data Register.");

mem_mapped!(special: BSP, 0xFFFA, "Backup Stack Pointer.");



mem_mapped!(special: PSR, PSR_ADDRESS, "Program Status Register.");

impl PSR {
    pub fn get_priority(&self) -> u8 {
        self.u8(8..10)
    }

    pub fn set_priority<I: InstructionInterpreter>(&mut self, interp: &mut I, priority: u8) {
        self.0 = (self.0 & (!WORD_MAX_VAL.word(8..10))) | (priority as Word);

        // Don't return a `WriteAttempt` since PSR accesses don't produce ACVs (and are hence infallible).
        self.write_current_value(interp);
    }

    pub fn n(&self) -> bool { self.bit(2) }
    pub fn z(&self) -> bool { self.bit(1) }
    pub fn p(&self) -> bool { self.bit(0) }
    pub fn get_cc(&self) -> (bool, bool, bool) { (self.n(), self.z(), self.p()) }

    pub fn set_cc<I: InstructionInterpreter>(&self, word: Word, interp: &mut I) {
        // `Word` is an unsigned type so we'll just assume 2's comp and check the
        // signed bit instead of using `.is_negative()`:
        let n: bool = (word >> ((core::mem::size_of::<Word>() * 8) - 1)) != 0;

        // z is easy enough to check for:
        let z: bool = word == 0;

        // if we're not negative or zero, we're positive:
        let p: bool = !(n | z);

        fn bit_to_word(bit: bool, left_shift: u32) -> u16 {
            (if bit { 1 } else { 0 }) << left_shift
        }

        let b = bit_to_word;

        self.0 = (self.0 & !(WORD_MAX_VAL.word(0..2))) | b(n, 2) | b(z, 1) | b(p, 0);

        // Don't return a `WriteAttempt` since PSR accesses are infallible.
        self.write_current_value(interp);
    }
}

mem_mapped!(special: MCR, MCR_ADDRESS, "Machine Control Register.");



// pub const KBDR: Addr = 0xFE02;

// // pub const KBSR: Addr = 0xFE00;
// pub const KBDR: Addr = 0xFE02;
// pub const DSR: Addr = 0xFE04;
// pub const DDR: Addr = 0xFE06;
// pub const BSP: Addr = 0xFFFA;
