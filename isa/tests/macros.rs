//! Test that the macros work when called from outside the crate.

#![cfg_attr(feature = "nightly-const", feature(const_if_match))]
#![cfg_attr(feature = "nightly-const", feature(const_panic))]

use lc3_isa::util::AssembledProgram;

use pretty_assertions::assert_eq;


#[cfg(feature = "nightly-const")]
const CONST_LOADABLE_TEST: [(lc3_isa::Addr, lc3_isa::Word); 28] = lc3_isa::loadable! {
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

#[cfg(feature = "nightly-const")]
const CONST_PROGRAM_TEST: [(lc3_isa::Word, bool); lc3_isa::ADDR_SPACE_SIZE_IN_WORDS] = lc3_isa::program! {
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

use lc3_isa::{Addr, Instruction::*, Reg::*, Word};

// TODO: actually go test const functionality with this...
#[test]
#[rustfmt::skip]
fn loadable_full() {

    // TODO: remove
    let LOADABLE: [(lc3_isa::Addr, lc3_isa::Word); 28] = lc3_isa::loadable! {
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

    assert_eq!(LOADABLE.len(), 28);
    assert_eq!(LOADABLE, [
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
        (0x300A, Ldr { dr: R0, base: R1, offset6: 31 }.into()),
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

    let program: [(lc3_isa::Word, bool); lc3_isa::ADDR_SPACE_SIZE_IN_WORDS] = lc3_isa::program! {
        .ORIG #0x3000  => is the program start;
        ADD R0, R0, R1 => you can use comments like this;
        ADD R1, R1, #0 => careful though there are things you cannot stick in these weird comments;
        AND R1, R2, R3 => like apostrophes and commas and leading numbers;
        AND R4, R5, #-0xF => also expressions and parens and most tokens like
                             periods and arrows;
        @l BRnzp @l; // Or you can always use good old Rust comments like this
        JMP R6;
        JSR #-1024;
        JSRR R2;

        // No labels unfortunately.
        @y LD R7, @y;
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

    let loadable: Vec<(Addr, Word)> = program
        .iter()
        .enumerate()
        .filter(|(_, (_, set))| {
            *set
        })
        .map(|(addr, (word, _))| {
            (addr as Addr, *word)
        })
        .collect();

    assert_eq!(loadable.len(), 28);
    assert_eq!(loadable.as_ref(), [
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
        (0x300A, Ldr { dr: R0, base: R1, offset6: 31 }.into()),
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
fn program_full_with_util() {

    let prog = lc3_isa::program! {
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

    let loadable: Vec<_> = Into::<AssembledProgram>::into(prog).into_iter().collect();

    // let mut loadable: [(Addr, Word); 28] = [(0, 0); 28];

    // use lc3_isa::util::AssembledProgram;

    // for (idx, (addr, word)) in Into::<AssembledProgram>::into(prog).into_iter().enumerate() {
    //     loadable[idx] = (addr, word);
    // }

    assert_eq!(loadable.len(), 28);
    assert_eq!(loadable, [
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
        (0x300A, Ldr { dr: R0, base: R1, offset6: 31 }.into()),
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
