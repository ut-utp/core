//! Macros for the LC-3 ISA.
//!
//! TODO!

// Note: talk about how this is only meant for writing const assembly (at compile time)
// as in, things like: `for reg in REGS { insn!{ADD reg, reg, R7 } }` won't work.
//
// Also warn about how this will (unfortunately) happily make unrepresentable instructions
// (i.e. we use an i16 to represent imm5s but this has no issue making an AddImm with an
// imm5 that won't fit in 5 bits).
#[macro_export]
macro_rules! insn {
    (ADD $dr:ident, $sr1:ident, $sr2:ident $(; $($extra:tt)*)?) => {
        $crate::Instruction::AddReg { dr: reg!($dr), sr1: reg!($sr1), sr2: reg!($sr2) }
    };
    (ADD $dr:ident, $sr1:ident, #$sr2:ident $(; $($extra:tt)*)?) => {
        $crate::Instruction::AddReg { dr: reg!($dr), sr1: reg!($sr1), sr2: reg!($sr2) }
    };
}

/// (TODO!)
///
/// ```compile_fail
/// reg!(R8);
/// ```
macro_rules! reg {
    (R0) => { $crate::Reg::R0 };
    (R1) => { $crate::Reg::R1 };
    (R2) => { $crate::Reg::R2 };
    (R3) => { $crate::Reg::R3 };
    (R4) => { $crate::Reg::R4 };
    (R5) => { $crate::Reg::R5 };
    (R6) => { $crate::Reg::R6 };
    (R7) => { $crate::Reg::R7 };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Instruction::{self, *}, Reg::{self, *}};

    #[test]
    fn test_regs() {
        assert_eq!(R0, reg!(R0));
        assert_eq!(R1, reg!(R1));
        assert_eq!(R2, reg!(R2));
        assert_eq!(R3, reg!(R3));
        assert_eq!(R4, reg!(R4));
        assert_eq!(R5, reg!(R5));
        assert_eq!(R6, reg!(R6));
        assert_eq!(R7, reg!(R7));
    }

    #[test]
    fn comments() {
        assert_eq!(insn!(ADD R0, R0, R0), insn!(ADD R0, R0, R0 ;));
        assert_eq!(insn!(ADD R0, R0, R0; One simple instruction ), insn!(ADD R0, R0, R0 ; <- Another simple instruction));
        assert_eq!(insn!(ADD R0, R0, R0; /* One simple instruction */ ), insn!(ADD R0, R0, R0 ; <- /*! Another simple instruction */));
    }

    #[test]
    fn add_reg() {
        assert_eq!(insn!(ADD R0, R1, R2), AddReg {dr: R0, sr1: R1, sr2: R2});
        assert_eq!(insn!(ADD R3, R0, R7), AddReg {dr: R3, sr1: R0, sr2: R7});

        assert_eq!(insn!(ADD R3, R4, R5), insn!(ADD R3, R4, R5));
        assert_ne!(insn!(ADD R3, R4, R5), insn!(ADD R3, R4, R5));
    }
}
