//! Macros for the LC-3 ISA.
//!
//! TODO!

// Note: talk about how this is only meant for writing const assembly (at compile time)
// as in, things like: `for reg in REGS { insn!{ADD reg, reg, R7 } }` won't work.
//
#[macro_export]
macro_rules! insn {
    (ADD $dr:ident, $sr1:ident, $sr2:ident $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_add_reg(reg!($dr), reg!($sr1), reg!($sr2))
    };
    (ADD $dr:ident, $sr1:ident, #$imm5:expr $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_add_imm(reg!($dr), reg!($sr1), $imm5)
    };
    // (ADD $dr:ident, $sr1:ident, x$imm5:tt $(=> $($extra:tt)*)?) => {
    //     $crate::Instruction::new_add_imm(reg!($dr), reg!($sr1), compile_fail!("Hex not yet supported; sorry."))
    // };

    (AND $dr:ident, $sr1:ident, $sr2:ident $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_and_reg(reg!($dr), reg!($sr1), reg!($sr1))
    };
    (AND $dr:ident, $sr1:ident, #$imm5:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_and_imm(reg!($dr), reg!($sr1), $imm5)
    };

    (BR #$offset9:expr $(=> $(extra:tt)*)?) => { insn!(BRnzp #$offset9) };
    (BRn #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(true, false, false, $offset9)
    };
    (BRz #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(false, true, false, $offset9)
    };
    (BRp #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(false, false, true, $offset9)
    };
    (BRnz #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(true, true, false, $offset9)
    };
    (BRnp #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(true, false, true, $offset9)
    };
    (BRzp #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(false, true, true, $offset9)
    };
    (BRnzp #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_br(true, true, true, $offset9)
    };

    (JMP $base:ident $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_jmp(reg!($base))
    };

    (JSR #$offset11:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_jsr($offset11)
    };

    (JSRR $base:ident $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_jsrr(reg!($base))
    };

    (LD $dr:ident, #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_ld(reg!($dr), $offset9)
    };

    (LDI $dr:ident, #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_lid(reg!($dr), $offset9)
    };

    (LDR $dr:ident, $base:ident, #$offset6:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_ldr(reg!($dr), reg!($base), $offset6)
    };

    (LEA $dr:ident, #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_lea(dr!($dr), $offset9)
    };

    (NOT $dr:ident, $sr:ident $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_not(dr!($dr), dr!($sr))
    };

    (RET $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_ret()
    };

    (RTI $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_rti()
    };

    (ST $sr:ident, #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_st(reg!($sr), $offset9)
    };

    (STI $sr:ident, #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_sti(reg!($sr), $offset9)
    };

    (STR $sr:ident, $base:ident, #$offset9:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_str(reg!($sr), reg!($base), $offset9)
    };

    (TRAP #$trapvec:expr $(=> $(extra:tt)*)?) => {
        $crate::Instruction::new_trap($trapvec)
    }
}

#[macro_export]
macro_rules! word {
    () => { 0 };
    // (.END) => {};
    (.FILL #$word:ident) => {
        Into::<$crate::Word>::into($word)
    };

    ($($other:tt)*) => {
        Into::<$crate::Word>::into(insn!($($other)*))
    }
}

/// (TODO!)
///
/// ```rust,compile_fail
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
    ($($other:tt)*) => { $($other)* };
}

#[cfg(test)]
mod tests {
    use crate::{Instruction::{self, *}, Reg::{self, *}};
    use core::convert::TryInto;

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

        assert_eq!(reg!(TryInto::<Reg>::try_into(7).unwrap()), R7);
    }

    #[test]
    fn comments() {
        assert_eq!(insn!(ADD R0, R0, R0), insn!(ADD R0, R0, R0 => yo));
        assert_eq!(insn!(ADD R0, R0, R0 => One simple instruction ), insn!(ADD R0, R0, R0 => <- Another simple instruction));
        assert_eq!(insn!(ADD R0, R0, R0 => /* One simple instruction */ ), insn!(ADD R0, R0, R0 =>  <- /*! Another simple instruction */));
        assert_eq!(insn!(ADD R0, R0, R0 => multiple
                lines
                are
                just
                fine
            ),
            insn!(ADD R0, R0, R0 =>  <- /*! Another simple instruction */)
        );
    }

    #[test]
    fn add_reg() {
        assert_eq!(insn!(ADD R0, R1, R2), AddReg { dr: R0, sr1: R1, sr2: R2 });
        assert_eq!(insn!(ADD R3, R0, R7), AddReg { dr: R3, sr1: R0, sr2: R7 });

        assert_eq!(insn!(ADD R3, R4, R5), insn!(ADD R3, R4, R5));
        assert_ne!(insn!(ADD R3, R4, R5), insn!(ADD R3, R4, R4));
    }

    #[test]
    fn add_imm() {
        assert_eq!(insn!(ADD R6, R7, #15), AddImm { dr: R6, sr1: R7, imm5: 15 });
        assert_eq!(insn!(ADD R6, R7, #-16), AddImm { dr: R6, sr1: R7, imm5: -16 });
        assert_eq!(insn!(ADD R6, R0, #0xF), AddImm { dr: R6, sr1: R0, imm5: 15 });
    }

    #[should_panic]
    #[test]
    fn add_imm_out_of_range() {
        let _ = insn!(ADD R0, R5, #16);
    }

    #[test]
    fn word() {
        assert_eq!(word!(ADD R0, R1, R2), AddReg { dr: R0, sr1: R1, sr2: R2 }.into());
        word!(); // Empty words are fine.
    }
}
