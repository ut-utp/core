//! Format impls. (TODO)

use core::fmt::{self, Display};
use super::isa::{Instruction, Reg};

impl Display for Reg {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Reg::*;
        write!(fmt, "R{}", match self {
            R0 => '0',
            R1 => '1',
            R2 => '2',
            R3 => '3',
            R4 => '4',
            R5 => '5',
            R6 => '6',
            R7 => '7',
        })
    }
}

