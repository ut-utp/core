//! Constants. (TODO)

use core::ops::Deref;
use lc3_isa::{Addr, Word, MCR as MCR_ADDRESS, PSR as PSR_ADDRESS};

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
        mem_mapped!(-; $name, $address, $comment);
    };

    (special: $name:ident, $address:expr, $comment:literal) => {
        mem_mapped!(+; $name, $address, $comment);
    };

    ($special:tt; $name:ident, $address:expr, $comment:literal) => {
        #[doc=$comment]
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

mem_mapped!(DSR, 0xFE04, "Display Status Register.");
mem_mapped!(DDR, 0xFE06, "Display Data Register.");

mem_mapped!(special: BSP, 0xFFFA, "Backup Stack Pointer.");
mem_mapped!(special: PSR, PSR_ADDRESS, "Program Status Register.");
mem_mapped!(special: MCR, MCR_ADDRESS, "Machine Control Register.");



// pub const KBDR: Addr = 0xFE02;

// // pub const KBSR: Addr = 0xFE00;
// pub const KBDR: Addr = 0xFE02;
// pub const DSR: Addr = 0xFE04;
// pub const DDR: Addr = 0xFE06;
// pub const BSP: Addr = 0xFFFA;
