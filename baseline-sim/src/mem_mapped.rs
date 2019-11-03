//! Constants. (TODO)

use core::ops::Deref;
use lc3_isa::{Addr, Word, MCR, PSR};

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

pub const KBSR: Addr = 0xFE00;
pub const KBDR: Addr = 0xFE02;
pub const DSR: Addr = 0xFE04;
pub const DDR: Addr = 0xFE06;
pub const BSP: Addr = 0xFFFA;
