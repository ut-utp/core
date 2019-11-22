//! Macros for the LC-3 ISA.
//!
//! TODO!

// Note: talk about how this is only meant for writing const assembly (at compile time)
// as in, things like: `for reg in REGS { insn!{ADD reg, reg, R7 } }` won't work.
//
#[macro_export]
macro_rules! insn {
    (ADD $dr:ident, $sr1:ident, $sr2:ident $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_add_reg($crate::reg!($dr), $crate::reg!($sr1), $crate::reg!($sr2))
    };
    (ADD $dr:ident, $sr1:ident, #$imm5:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_add_imm($crate::reg!($dr), $crate::reg!($sr1), $imm5)
    };

    (AND $dr:ident, $sr1:ident, $sr2:ident $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_and_reg($crate::reg!($dr), $crate::reg!($sr1), $crate::reg!($sr2))
    };
    (AND $dr:ident, $sr1:ident, #$imm5:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_and_imm($crate::reg!($dr), $crate::reg!($sr1), $imm5)
    };

    (BR #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => { $crate::insn!(BRnzp #$offset9) };
    (BRn #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(true, false, false, $offset9)
    };
    (BRz #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(false, true, false, $offset9)
    };
    (BRp #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(false, false, true, $offset9)
    };
    (BRnz #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(true, true, false, $offset9)
    };
    (BRnp #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(true, false, true, $offset9)
    };
    (BRzp #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(false, true, true, $offset9)
    };
    (BRnzp #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_br(true, true, true, $offset9)
    };

    (JMP $base:ident $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_jmp($crate::reg!($base))
    };

    (JSR #$offset11:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_jsr($offset11)
    };

    (JSRR $base:ident $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_jsrr($crate::reg!($base))
    };

    (LD $dr:ident, #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_ld($crate::reg!($dr), $offset9)
    };

    (LDI $dr:ident, #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_ldi($crate::reg!($dr), $offset9)
    };

    (LDR $dr:ident, $base:ident, #$offset6:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_ldr($crate::reg!($dr), $crate::reg!($base), $offset6)
    };

    (LEA $dr:ident, #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_lea($crate::reg!($dr), $offset9)
    };

    (NOT $dr:ident, $sr:ident $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_not($crate::reg!($dr), $crate::reg!($sr))
    };

    (RET $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_ret()
    };

    (RTI $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_rti()
    };

    (ST $sr:ident, #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_st($crate::reg!($sr), $offset9)
    };

    (STI $sr:ident, #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_sti($crate::reg!($sr), $offset9)
    };

    (STR $sr:ident, $base:ident, #$offset9:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_str($crate::reg!($sr), $crate::reg!($base), $offset9)
    };

    (TRAP #$trapvec:expr $(,)? $(=> $($extra:tt)*)?) => {
        $crate::Instruction::new_trap($trapvec)
    }
}

#[macro_export]
macro_rules! word {
    () => { 0 };
    // (.END) => {};
    ($(.)? FILL #$word:expr $(=> $($extra:tt)*)?) => {
        // Once const-fn arrives:
        Into::<$crate::Word>::into($word)
    };

    ($(.)? BLKW #$word:expr $(=> $($extra:tt)*)?) => {
        panic!("Sorry! .BLKW isn't supported. Try `lc3_isa::string!()`?");
    };

    (GETC $(=> $($extra:tt)*)?) => { $crate::word!(TRAP #0x20) };
    (OUT $(=> $($extra:tt)*)?) => { $crate::word!(TRAP #0x21) };
    (PUTS $(=> $($extra:tt)*)?) => { $crate::word!(TRAP #0x22) };
    (IN $(=> $($extra:tt)*)?) => { $crate::word!(TRAP #0x23) };
    (HALT $(=> $($extra:tt)*)?) => { $crate::word!(TRAP #0x25) };

    (NOP $(=> $($extra:tt)*)?) => { $crate::word!(BR #0) };

    ($($other:tt)*) => {
        Into::<$crate::Word>::into($crate::insn!($($other)*))
    }
}

/// TODO!
///
/// Supports one or two byte UTF-8 characters only.
// TODO: maybe don't panic?
// Ideally we'd do all the checks at compile time but const functions are pretty
// far away from being able to do that so maybe we should go turn this into a
// proc macro once we've got some more time.
// proc macro it is! (TODO!) We can spit out a proper array (like loadable!)
// instead of a macro.
#[macro_export]
macro_rules! string {
    ($addr:expr, $string:literal) => {
        {
            ($string).chars().map(|c|) {
                match c.len_utf8() {
                    3..=4 => panic!("Can't represent `{}` in <= 2 bytes!", c);
                    2 => {
                        let mut buf: [u8; 2] = [0, 0];
                        c.encode_utf8(&mut buf);
                    }
                }
            }

            let mut arr = [];
            let mut addr = ($addr);

            for c in $string.chars() {

                addr += 1;
            }

            arr
        }
    };
}

#[macro_export]
macro_rules! loadable {
    // Note: `$(=> $($_a:ident$($_b:literal)?)*)?` is a bad approximation of comments, but c'est la vie
    // Note: this permits dots before instructions and allows dots to be omitted before directives (i.e. both `.ADD` and `FILL` are allowed), but w/e
    (.ORIG #$orig:expr $(=> $($_oa:ident$($_ob:literal)?)*)?; $($(.)? $op:ident $($regs:ident),* $(,)? $(#$num:expr)? $(=> $($_a:ident$($_b:literal)?)*)?;)*) => {
        {
            #[allow(mutable)]
            let mut _addr: $crate::Addr = $orig;

            [$(
                ({_addr += 1; _addr - 1}, $crate::word!($op $($regs,)* $(#$num)*)),
            )*]

        }
    };
}

#[macro_export]
// Note: Supports multiple `.ORIG`s.
// Note: Same bad approximation of comments. It's fine.
// Note: The thing that translates from this to loadables doesn't work in
//       const contexts yet.
macro_rules! program {
    (.ORIG #$orig:expr $(=> $($_oa:ident$($_ob:literal)?)*)?; $($rest:tt)*) => {
        {
            // On no_std, since we can't use Vec and other heap backed types to
            // make a dynamically sized thing, we'll just make something as big
            // as the entire address space and use that to generate the (addr,
            // word) pairs.
            //
            // This is a little silly since this almost certainly will actually
            // run at compile time on the host (not subject to no_std
            // restrictions) but:
            //   - vec::push is not yet a const function (vec::new is on 1.39+
            //     but, iiuc that's because it doesn't actually allocate heap
            //     space).
            #[allow(mutable)]
            let mut _addr: $crate::Addr;

            #[allow(mutable)]
            let mut _mem: [($crate::Word, bool); $crate::ADDR_SPACE_SIZE_IN_WORDS] = [(0, false); $crate::ADDR_SPACE_SIZE_IN_WORDS];

            $crate::program!(%%contd, _addr, _mem | .ORIG #$orig; $($rest)*);
        }
    };

    (%%contd, $addr:ident, $mem:ident | .ORIG #$orig:expr $(=> $($_oa:ident$($_ob:literal)?)*)?; $($rest:tt)*) => {
        $addr = $orig;

        $crate::program!(%%contd, $addr, $mem | $($rest)*);
    };

    (%%contd, $addr:ident, $mem:ident | $name:ident #$orig:expr $(=> $($_oa:ident$($_ob:literal)?)*)?; $($rest:tt)*) => {
        println!("foo");
    };

    // The end!
    (%%contd, $addr:ident, $mem:ident |) => {

    };
}

#[macro_export]
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
    const REG_R7: Reg = reg!(R7);

    use crate::{
        Addr,
        Instruction::{self, *},
        Reg::{self, *},
        Word,
    };
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
        assert_eq!(
            insn!(ADD R0, R0, R0 => One simple instruction ),
            insn!(ADD R0, R0, R0 => <- Another simple instruction)
        );
        assert_eq!(
            insn!(ADD R0, R0, R0 => /* One simple instruction */ ),
            insn!(ADD R0, R0, R0 =>  <- /*! Another simple instruction */)
        );
        assert_eq!(
            insn!(ADD R0, R0, R0 => multiple
                lines
                are
                just
                fine
            ),
            insn!(ADD R0, R0, R0 =>  <- /*! Another simple instruction */)
        );
    }

    #[test]
    fn misc() {
        let insn =
            insn!(AND R0, R0, R0, => Unfortunately we'll take trailing commas, but don't do this!);

        assert_eq!(insn, insn!(AND R0, R0, R0));

        word!(.FILL #0x3000 as Word);
    }

    #[test]
    fn add_reg() {
        assert_eq!(
            insn!(ADD R0, R1, R2),
            AddReg {
                dr: R0,
                sr1: R1,
                sr2: R2
            }
        );
        assert_eq!(
            insn!(ADD R3, R0, R7),
            AddReg {
                dr: R3,
                sr1: R0,
                sr2: R7
            }
        );

        assert_eq!(insn!(ADD R3, R4, R5), insn!(ADD R3, R4, R5));
        assert_ne!(insn!(ADD R3, R4, R5), insn!(ADD R3, R4, R4));
    }

    #[test]
    fn add_imm() {
        assert_eq!(
            insn!(ADD R6, R7, #15),
            AddImm {
                dr: R6,
                sr1: R7,
                imm5: 15
            }
        );
        assert_eq!(
            insn!(ADD R6, R7, #-16),
            AddImm {
                dr: R6,
                sr1: R7,
                imm5: -16
            }
        );
        assert_eq!(
            insn!(ADD R6, R0, #0xF),
            AddImm {
                dr: R6,
                sr1: R0,
                imm5: 15
            }
        );
    }

    #[should_panic]
    #[test]
    fn add_imm_out_of_range() {
        let _ = insn!(ADD R0, R5, #16);
    }

    #[test]
    fn word() {
        assert_eq!(
            word!(ADD R0, R1, R2),
            AddReg {
                dr: R0,
                sr1: R1,
                sr2: R2
            }
            .into()
        );
        word!(); // Empty words are fine.
    }

    #[test]
    fn loadable_empty() {
        let prog: [(Addr, Word); 0] = loadable! {
            .ORIG #0x3000u16;
        };

        assert_eq!(prog, []);
    }

    #[test]
    #[rustfmt::skip]
    fn loadable_full() {
        let prog = loadable! {
            .ORIG #0x3000  => is the program start;
            ADD R0, R0, R1 => you can use comments like this;
            ADD R1, R1, #0 => careful though there are things you cannot stick in these weird comments;
            AND R1, R2, R3 => like apostrophes and commas and leading numbers;
            AND R4, R5, #-0xF => also expressions and parens and most tokens like
                                 periods and arrows;
            BRnzp #-1; // Or you can always use good old Rust comments like this
            JMP R6;
            JSR #-1024;
            JSRR R2;

            // No labels unfortunately.
            LD R7, #-1;
            LDI R4, #255;
            LDR R0, R1, #31;
            LEA R0, #12;

            // After all this isn't an assembler.
            NOT R2, R3;
            RET;
            RTI;

            // So, make good use of comments if you're going to write things this way.
            ST R2, #-45;
            STI R7, #3;
            STR R2, R0, #-32;

            TRAP #0x25;

            ADD R0, R2, #0;
            OUT;
            PUTS;

            AND R0, R0, #0;
            GETC;

            AND R0, R0, #0;
            IN;

            HALT;

            .FILL #0x23u16;
        };

        assert_eq!(prog.len(), 28);
        assert_eq!(prog, [
            (0x3000, AddReg { dr: R0, sr1: R0, sr2: R1 }.into()),
            (0x3001, AddImm { dr: R1, sr1: R1, imm5: 0 }.into()),
            (0x3002, AndReg { dr: R1, sr1: R2, sr2: R3 }.into()),
            (0x3003, AndImm { dr: R4, sr1: R5, imm5: -0xF }.into()),
            (0x3004, Br { n: true, z: true, p: true, offset9: -1 }.into()),
            (0x3005, Jmp { base: R6 }.into()),
            (0x3006, Jsr { offset11: -1024 }.into()),
            (0x3007, Jsrr { base: R2}.into()),
            (0x3008, Ld { dr: R7, offset9: -1 }.into()),
            (0x3009, Ldi { dr: R4, offset9: 255 }.into()),
            (0x300A, Ldr { dr: R0, base: R0, offset6: 511 }.into()),
            (0x300B, Lea { dr: R0, offset9: 12 }.into()),
            (0x300C, Not { dr: R2, sr: R3 }.into()),
            (0x300D, Ret.into()),
            (0x300E, Rti.into()),
            (0x300F, St { sr: R2, offset9: -45 }.into()),
            (0x3010, Sti { sr: R7, offset9: 3 }.into()),
            (0x3011, Str { sr: R2, base: R0, offset6: -32 }.into()),
            (0x3012, Trap { trapvec: 0x25 }.into()),
            (0x3013, AddImm { dr: R0, sr1: R2, imm5: 0 }.into()),
            (0x3014, Trap { trapvec: 0x21 }.into()),
            (0x3015, Trap { trapvec: 0x22 }.into()),
            (0x3016, AndImm { dr: R0, sr1: R0, imm5: 0 }.into()),
            (0x3017, Trap { trapvec: 0x20 }.into()),
            (0x3018, AndImm { dr: R0, sr1: R0, imm5: 0 }.into()),
            (0x3019, Trap { trapvec: 0x23 }.into()),
            (0x301A, Trap { trapvec: 0x25 }.into()),
            (0x301B, 0x23),
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn program_full() {
        let prog = program! {
            .ORIG #0x3000  => is the program start;
            // ADD R0, R0, R1 => you can use comments like this;
            // ADD R1, R1, #0 => careful though there are things you cannot stick in these weird comments;
            // AND R1, R2, R3 => like apostrophes and commas and leading numbers;
            // AND R4, R5, #-0xF => also expressions and parens and most tokens like
            //                      periods and arrows;
            // BRnzp #-1; // Or you can always use good old Rust comments like this
            // JMP R6;
            // JSR #-1024;
            // JSRR R2;

            // // No labels unfortunately.
            // LD R7, #-1;
            // LDI R4, #255;
            // LDR R0, R1, #31;
            // LEA R0, #12;

            // // After all this isn't an assembler.
            // NOT R2, R3;
            // RET;
            // RTI;

            // // So, make good use of comments if you're going to write things this way.
            // ST R2, #-45;
            // STI R7, #3;
            // STR R2, R0, #-32;

            // TRAP #0x25;

            // ADD R0, R2, #0;
            // OUT;
            // PUTS;

            // AND R0, R0, #0;
            // GETC;

            // AND R0, R0, #0;
            // IN;

            // HALT;

            // .FILL #0x23u16;
        };

        // assert_eq!(prog.len(), 28);
        // assert_eq!(prog, [
        //     (0x3000, AddReg { dr: R0, sr1: R0, sr2: R1 }.into()),
        //     (0x3001, AddImm { dr: R1, sr1: R1, imm5: 0 }.into()),
        //     (0x3002, AndReg { dr: R1, sr1: R2, sr2: R3 }.into()),
        //     (0x3003, AndImm { dr: R4, sr1: R5, imm5: -0xF }.into()),
        //     (0x3004, Br { n: true, z: true, p: true, offset9: -1 }.into()),
        //     (0x3005, Jmp { base: R6 }.into()),
        //     (0x3006, Jsr { offset11: -1024 }.into()),
        //     (0x3007, Jsrr { base: R2}.into()),
        //     (0x3008, Ld { dr: R7, offset9: -1 }.into()),
        //     (0x3009, Ldi { dr: R4, offset9: 255 }.into()),
        //     (0x300A, Ldr { dr: R0, base: R0, offset6: 511 }.into()),
        //     (0x300B, Lea { dr: R0, offset9: 12 }.into()),
        //     (0x300C, Not { dr: R2, sr: R3 }.into()),
        //     (0x300D, Ret.into()),
        //     (0x300E, Rti.into()),
        //     (0x300F, St { sr: R2, offset9: -45 }.into()),
        //     (0x3010, Sti { sr: R7, offset9: 3 }.into()),
        //     (0x3011, Str { sr: R2, base: R0, offset6: -32 }.into()),
        //     (0x3012, Trap { trapvec: 0x25 }.into()),
        //     (0x3013, AddImm { dr: R0, sr1: R2, imm5: 0 }.into()),
        //     (0x3014, Trap { trapvec: 0x21 }.into()),
        //     (0x3015, Trap { trapvec: 0x22 }.into()),
        //     (0x3016, AndImm { dr: R0, sr1: R0, imm5: 0 }.into()),
        //     (0x3017, Trap { trapvec: 0x20 }.into()),
        //     (0x3018, AndImm { dr: R0, sr1: R0, imm5: 0 }.into()),
        //     (0x3019, Trap { trapvec: 0x23 }.into()),
        //     (0x301A, Trap { trapvec: 0x25 }.into()),
        //     (0x301B, 0x23),
        // ]);
    }

    const A_SIMPLE_INSN: Instruction = insn!(ADD R0, R0, R0);

    // TODO: make this work in const contexts!
    // const COMPLEX_LOADABLE: [(Addr, Word); 28] = loadable! {
    //     .ORIG #0x3000  => is the program start;
    //     ADD R0, R0, R1 => you can use comments like this;
    //     ADD R1, R1, #0 => careful though there are things you cannot stick in these weird comments;
    //     AND R1, R2, R3 => like apostrophes and commas and leading numbers;
    //     AND R4, R5, #-0xF => also expressions and parens and most tokens like
    //                          periods and arrows;
    //     BRnzp #-1; // Or you can always use good old Rust comments like this
    //     JMP R6;
    //     JSR #-1024;
    //     JSRR R2;

    //     // No labels unfortunately.
    //     LD R7, #-1;
    //     LDI R4, #255;
    //     LDR R0, R1, #31;
    //     LEA R0, #12;

    //     // After all this isn't an assembler.
    //     NOT R2, R3;
    //     RET;
    //     RTI;

    //     // So, make good use of comments if you're going to write things this way.
    //     ST R2, #-45;
    //     STI R7, #3;
    //     STR R2, R0, #-32;

    //     TRAP #0x25;

    //     ADD R0, R2, #0;
    //     OUT;
    //     PUTS;

    //     AND R0, R0, #0;
    //     GETC;

    //     AND R0, R0, #0;
    //     IN;

    //     HALT;

    //     .FILL #0x23u16;
    // };

    // #[test]
    // #[rustfmt::skip]
    // fn const_context() {
    //     assert_eq!(REG_R7, R7);
    //     assert_eq!(A_SIMPLE_INSN, insn!(ADD R0, R0, R0 => yo));
    //     assert_eq!(COMPLEX_LOADABLE, [
    //         (0x3000, AddReg { dr: R0, sr1: R0, sr2: R1 }.into()),
    //         (0x3001, AddImm { dr: R1, sr1: R1, imm5: 0 }.into()),
    //         (0x3002, AndReg { dr: R1, sr1: R2, sr2: R3 }.into()),
    //         (0x3003, AndImm { dr: R4, sr1: R5, imm5: -0xF }.into()),
    //         (0x3004, Br { n: true, z: true, p: true, offset9: -1 }.into()),
    //         (0x3005, Jmp { base: R6 }.into()),
    //         (0x3006, Jsr { offset11: -1024 }.into()),
    //         (0x3007, Jsrr { base: R2}.into()),
    //         (0x3008, Ld { dr: R7, offset9: -1 }.into()),
    //         (0x3009, Ldi { dr: R4, offset9: 255 }.into()),
    //         (0x300A, Ldr { dr: R0, base: R0, offset6: 511 }.into()),
    //         (0x300B, Lea { dr: R0, offset9: 12 }.into()),
    //         (0x300C, Not { dr: R2, sr: R3 }.into()),
    //         (0x300D, Ret.into()),
    //         (0x300E, Rti.into()),
    //         (0x300F, St { sr: R2, offset9: -45 }.into()),
    //         (0x3010, Sti { sr: R7, offset9: 3 }.into()),
    //         (0x3011, Str { sr: R2, base: R0, offset6: -32 }.into()),
    //         (0x3012, Trap { trapvec: 0x25 }.into()),
    //         (0x3013, AddImm { dr: R0, sr1: R2, imm5: 0 }.into()),
    //         (0x3014, Trap { trapvec: 0x21 }.into()),
    //         (0x3015, Trap { trapvec: 0x22 }.into()),
    //         (0x3016, AndImm { dr: R0, sr1: R0, imm5: 0 }.into()),
    //         (0x3017, Trap { trapvec: 0x20 }.into()),
    //         (0x3018, AndImm { dr: R0, sr1: R0, imm5: 0 }.into()),
    //         (0x3019, Trap { trapvec: 0x23 }.into()),
    //         (0x301A, Trap { trapvec: 0x25 }.into()),
    //         (0x301B, 0x23),
    //     ]);
    // }
}

#[cfg(test)]
mod misc {
    use crate::Instruction::*;

    #[test]
    fn br_encoding_insn_macro() {
        if let Br { offset9, .. } = insn!(BRnzp #-1) {
            assert_eq!(offset9, -1);
        } else { assert!(false); }

        if let Br { offset9, .. } = insn!(BRnzp #90) {
            assert_eq!(offset9, 90);
        } else { assert!(false); }

        if let Br { offset9, .. } = insn!(BRnzp #0) {
            assert_eq!(offset9, 0);
        } else { assert!(false); }
    }
}
