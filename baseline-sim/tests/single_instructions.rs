extern crate lc3_test_infrastructure as lti;

use lti::{insn, Addr, Instruction, Reg, Word};
use lti::{MemoryShim, PeripheralsShim, PeripheralInterruptFlags};

#[cfg(test)]
mod lc3tools {
    use super::*;
    use lti::assert_eq;
    use lti::{lc3tools_tester, with_larger_stack};

    macro_rules! lc3_sequence {
        ($(|$panics:literal|)? $name:ident, insns: [ $({ $($insn:tt)* }),* ], lc3_insns: [ $($lc3_insn:expr),* ]) => {
        $(#[doc = $panics] #[should_panic])?
        #[test]
        fn $name() { with_larger_stack(/*Some(stringify!($name).to_string())*/ None, || {



            #[allow(unused_mut)]
            let mut insns: Vec<Instruction> = Vec::new();
            $(insns.push(insn!($($insn)*));)*

            #[allow(unused_mut)]
            let mut lc3_insns: Vec<String> = Vec::new();
            $(
                lc3_insns.push($lc3_insn);
            )*

            let flags = PeripheralInterruptFlags::new();

            lc3tools_tester::<MemoryShim, PeripheralsShim, _, _>(
                Vec::new(),
                insns,
                lc3_insns,
                (|_p| {}), // (no-op)
                (|_p| {}), // (no-op)
                &flags,
                &None
            );
        })}};
    }


mod prog {
    use super::*;

    lc3_sequence!{
        add_one,
        insns: [ { ADD R0, R0, #1 }, { ADD R1, R1, #1 }, { ADD R2, R2, #1 }, { ADD R3, R3, #1 }, { ADD R4, R4, #1 }, { ADD R5, R5, #1 }, { ADD R6, R6, #1 }, { ADD R7, R7, #1 } ],
        lc3_insns: [ "add r0, r0, #1".to_string(), "add r1, r1, #1".to_string(), "add r2, r2, #1".to_string(), "add r3, r3, #1".to_string(), "add r4, r4, #1".to_string(), "add r5, r5, #1".to_string(), "add r6, r6, #1".to_string(), "add r7, r7, #1".to_string() ]
    }

    lc3_sequence!{
        set_memory,
        insns: [ { ADD R0, R0, #1 }, { ST R0, #5 }, { LD R1, #4} ],
        lc3_insns: [ "add r0, r0, #1".to_string(), "st r0, #5".to_string(), "ld r1, #4".to_string()]
    }
    lc3_sequence!{
        add_and_set,
        insns: [ { ADD R0, R0, #1 }, { AND R0, R1, R0 }, { ADD R2, R2, #1 }, { ADD R0, R2, R2 }, { AND R0, R0, R2 }, { ADD R5, R5, #1 }, { LD R5, #10 }, { ADD R7, R7, #1 }, { ST R0, #5 }, { LD R1, #4} ],
        lc3_insns: [ "add r0, r0, #1".to_string(),"and r0, r1, r0".to_string(), "add r2, r2, #1".to_string(), "add r0, r2, r2".to_string(), "and r0, r0, r2".to_string(), "add r5, r5, #1".to_string(), "ld r5, #10".to_string(), "add r7, r7, #1".to_string(), "st r0, #5".to_string(), "ld r1, #4".to_string()]
    }



}


}

#[cfg(test)]
mod single_instructions {
    use super::*;
    use Reg::*;

    use lti::assert_eq;
    use lti::{interp_test_runner, with_larger_stack};

    // Test that the instructions work
    // Test that the unimplemented instructions do <something>

    macro_rules! sequence {
        ($(|$panics:literal|)? $name:ident, insns: [ $({ $($insn:tt)* }),* ], steps: $steps:expr, ending_pc: $pc:literal, regs: { $($r:tt: $v:expr),* }, memory: { $($addr:literal: $val:expr),* }) => {
        $(#[doc = $panics] #[should_panic])?
        #[test]
        fn $name() { with_larger_stack(/*Some(stringify!($name).to_string())*/ None, || {

            #[allow(unused_mut)]
            let mut regs: [Option<Word>; Reg::NUM_REGS] = [None, None, None, None, None, None, None, None];
            $(regs[Into::<u8>::into($r) as usize] = Some($v);)*

            #[allow(unused_mut)]
            let mut checks: Vec<(Addr, Word)> = Vec::new();
            $(checks.push(($addr, $val));)*

            #[allow(unused_mut)]
            let mut insns: Vec<Instruction> = Vec::new();
            $(insns.push(insn!($($insn)*));)*

            let flags = PeripheralInterruptFlags::new();

            interp_test_runner::<MemoryShim, PeripheralsShim, _, _>(
                Vec::new(),
                insns,
                $steps,
                regs,
                Some($pc),
                checks,
                (|_p| {}), // (no-op)
                (|_p| {}), // (no-op)
                &flags,
                None
            );
        })}};
    }

    // TODO: test macro like above but takes a program instead of a sequence of instructions (and uses the loadable! macro or the program macro).
    /////////
    // ADD //
    /////////
    mod add {
        use super::*;

        sequence! {
            add_nop,
            insns: [ { ADD R0, R0, #0 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: 0 },
            memory: {}
        }

        sequence! {
            add_imm_pos,
            insns: [ { ADD R0, R0, #1 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: 1 },
            memory: {}
        }

        sequence! {
            add_imm_neg,
            insns: [ { ADD R0, R0, #-1 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: 0xffff },
            memory: {}
        }

        sequence! {
            add_reg,
            insns: [
                { ADD R0, R0, #1 },
                { ADD R1, R1, #2 },
                { ADD R2, R0, R1 }
            ],
            steps: Some(3),
            ending_pc: 0x3003,
            regs: { R0: 1, R1: 2, R2: 3 },
            memory: {}
        }

        sequence! {
            add_reg_uninitialized,
            insns: [
                { ADD R0, R5, R5 }
            ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: 0, R1: 0, R2: 0 },
            memory: {}
        }
    }

    /////////
    // AND //
    /////////
    mod and {
        use super::*;

        sequence! {
            and_0_nop,
            insns: [ { AND R0, R0, #0 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: 0 },
            memory: {}
        }

        sequence! {
            and_1_nop,
            insns: [ { AND R0, R0, #1 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: 0 },
            memory: {}
        }

        sequence! {
            and_0_imm,
            insns: [
                { ADD R0, R0, #1 },
                { AND R0, R0, #0 }
            ],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: { R0: 0 },
            memory: {}
        }

        sequence! {
            and_1_imm,
            insns: [
                { ADD R0, R0, #1 },
                { AND R0, R0, #1 }
            ],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: { R0: 1 },
            memory: {}
        }

        sequence! {
            and_0_reg,
            insns: [
                { ADD R0, R0, #1 },
                { AND R0, R0, R1 }
            ],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: { R0: 0 },
            memory: {}
        }

        sequence! {
            and_1_reg,
            insns: [
                { ADD R0, R0, #1 },
                { ADD R1, R1, #1 },
                { AND R2, R0, R1 }
            ],
            steps: Some(3),
            ending_pc: 0x3003,
            regs: { R0: 1, R1: 1, R2: 1 },
            memory: {}
        }
    }

    ////////
    // BR //
    ////////
    mod br {
        use super::*;

        sequence! {
            branch_self,
            insns: [ { BRnzp #-1 } ],
            steps: Some(20),
            ending_pc: 0x3000,
            regs: { },
            memory: {}
        }

        sequence! {
            no_op,
            insns: [ { BRnzp #0 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { },
            memory: {}
        }

        sequence! {
            branch_simple,
            insns: [ { AND R0, R0, #0 }, { BRz #3 } ],
            steps: Some(2),
            ending_pc: 0x3005,
            regs: {},
            memory: {}
        }

        sequence! {
            |"should fail"|
            no_op_fail,
            insns: [ { BRnzp #2 } ],
            steps: Some(1),
            ending_pc: 0x3000,
            regs: {},
            memory: {}
        }
    }

    /////////
    // JMP //
    /////////
    mod jmp {
        use super::*;

        sequence! {
            jmp_0,
            insns: [ { JMP R0 } ],
            steps: Some(1),
            ending_pc: 0x0000,
            regs: {},
            memory: {}
        }

        sequence! {
            jmp_1,
            insns: [ { ADD R0, R0, #1 }, { JMP R0 } ],
            steps: Some(2),
            ending_pc: 0x0001,
            regs: {},
            memory: {}
        }
    }

    /////////
    // JSR //
    /////////
    mod jsr {
        use super::*;

        sequence! {
            jsr_2,
            insns: [ { JSR #2 } ],
            steps: Some(1),
            ending_pc: 0x3003,
            regs: { R7: 0x3001 },
            memory: {}
        }

        sequence! {
            jsr_neg,
            insns: [ { JSR #-10 } ],
            steps: Some(1),
            ending_pc: 0x2FF7,
            regs: { R7: 0x3001 },
            memory: {}
        }
    }

    //////////
    // JSRR //
    //////////
    mod jsrr {
        use super::*;

        sequence! {
            jsrr_0,
            insns: [ { JSRR R0 } ],
            steps: Some(1),
            ending_pc: 0x0000,
            regs: { R7: 0x3001 },
            memory: {}
        }

        sequence! {
            jsrr_1,
            insns: [ { ADD R0, R0, #1 }, { JSRR R0 } ],
            steps: Some(2),
            ending_pc: 0x0001,
            regs: { R7: 0x3002 },
            memory: {}
        }

        // In an incorrect implementation (if we don't store the PC in an
        // intermediary 'register'), this could happen:
        //   R7 <- PC // base register value overwritten!!
        //   PC <- R7
        //
        // When R7 isn't the register that's used with JSRR, this works:
        //   R7 <- PC
        //   PC <- R1
        //
        // On incorrect implementations, the below should fail. R7 will be
        // 0x3002 as expected but the PC will be 0x3002 (making it seem as
        // though the JSRR never really happened).
        sequence! {
            jsrr_r7,
            insns: [ { ADD R7, R7, #0x08 }, { JSRR R7 } ],
            steps: Some(2),
            ending_pc: 0x0008,
            regs: { R7: 0x3002 },
            memory: {}
        }
    }

    ////////
    // LD //
    ////////
    mod ld {
        use super::*;

        sequence! {
            ld_self,
            insns: [ { LD R0, #-1 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: Instruction::Ld{dr: R0, offset9: -1}.into() },
            memory: {}
        }

        sequence! {
            ld_0,
            insns: [ { LD R0, #0 }, { ADD R0, R0, R0 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: Instruction::AddReg{dr: R0, sr1: R0, sr2: R0}.into() },
            memory: {}
        }

        sequence! {
            ld_pos,
            insns: [ { LD R0, #1 }, { AND R0, R0, R0 }, { ADD R0, R0, R0 } ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: { R0: Instruction::AddReg{dr: R0, sr1: R0, sr2: R0}.into() },
            memory: {}
        }
    }

    /////////
    // NOT //
    /////////
    mod not {
        use super::*;

        sequence! { // take 0
            not_0,
            insns: [ { ADD R0, R0, #0 }, { NOT R0, R0} ],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 0xffff},
            memory: {}
        }

        sequence! {  // take a positive number
            not_1,
            insns: [ { ADD R0, R0, #1 }, { NOT R0, R0 } ],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 0xfffe},
            memory: {}
        }

        sequence! { // take a negative number
            not_neg,
            insns: [ { ADD R0, R0, #-1 }, { NOT R0, R0 } ],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 0},
            memory: {}
        }
    }

    ////////
    // ST //
    ////////
    mod st {
        use super::*;

        sequence! { // take 0
            st_0,
            insns: [ { ADD R0, R0, #0}, {ST R0, #16}],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 0},
            memory: {0x3012: 0}
        }

        sequence! { // take 1
            st_1,
            insns: [ { ADD R0, R0, #1}, {ST R0, #16}],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 1},
            memory: {0x3012: 1}
        }

        sequence! { // take -1
            st_neg,
            insns: [ { ADD R0, R0, #-1}, {ST R0, #16}],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 0xffff},
            memory: {0x3012: 0xffff}
        }

        sequence! { // store behind
            st_neg_offset,
            insns: [ { ADD R0, R0, #1}, {ST R0, #-16}],
            steps: Some(2),
            ending_pc: 0x3002,
            regs: {R0: 1},
            memory: {0x2FF2: 1}
        }
    }

    /////////
    // RET //
    /////////
    mod ret {
        use super::*;

        sequence! {
            ret_2,
            insns: [ { JSR #2 }, {ADD R0, R0, #0}, {ADD R0, R0, #0}, { RET } ],
            steps: Some(2),
            ending_pc: 0x3001,
            regs: { R7: 0x3001 },
            memory: {}
        }

        sequence! {
            ret_pos_neg,
            insns: [ { JSR #1 }, {RET}, {ADD R0, R0, #0}, {ADD R0, R0, #0}, { JSR #-4 } ],
            steps: Some(5),
            ending_pc: 0x3005,
            regs: { R7: 0x3005 },
            memory: {}
        }

        // load the return into a register -> store it somewhere -> jump there
        sequence! {
            ret_neg_pos,
            insns: [{LD R0, #3}, {ST R0, #-2}, {ADD R0, R0, #0}, { JSR #-4 }, {RET},  { JSR #-4 } ],
            steps: Some(5),
            ending_pc: 0x3004,
            regs: { R7: 0x3004 },
            memory: {}
        }
        // not sure how to test negative returns...
        // Need to store return in a previous address
        // then can jump to there... ?
    }

    /////////
    // STI //
    /////////
    mod sti {
        use super::*;

        sequence! {
            sti_pos,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {ST R0, #2}, {STI R1, #1}],
            steps: Some(4),
            ending_pc: 0x3004,
            regs: {},
            memory: {0x3011: 1}
        }

        sequence! {
            sti_zero,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {ST R0, #1}, {STI R1, #0}],
            steps: Some(4),
            ending_pc: 0x3004,
            regs: {},
            memory: {0x3011: 1}
        }

        sequence! {
            sti_neg,
            insns: [ { LEA R0, #-1}, {ADD R1, R1, #1}, {ST R0, #-1}, {STI R1, #-2}],
            steps: Some(4),
            ending_pc: 0x3004,
            regs: {},
            memory: {0x3000: 1}
        }
    }

    /////////
    // STR //
    /////////
    mod str {
        use super::*;

        sequence! {
            str_pos,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {STR R1, R0, #1}],
            steps: Some(3),
            ending_pc: 0x3003,
            regs: {},
            memory: {0x3012: 1}
        }

        sequence! {
            str_zero,
            insns: [ { LEA R0, #-1}, {ADD R1, R1, #1}, {STR R1, R0, #0}],
            steps: Some(3),
            ending_pc: 0x3003,
            regs: {},
            memory: {0x3000: 1}
        }

        sequence! {
            str_neg,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {STR R1, R0, #-1}],
            steps: Some(3),
            ending_pc: 0x3003,
            regs: {},
            memory: {0x3010: 1}
        }
    }

    /////////
    // LDR //
    /////////
    mod ldr {
        use super::*;

        sequence! {
            ldr_pos,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {STR R1, R0, #1}, {LDR R2, R0, #1}],
            steps: Some(4),
            ending_pc: 0x3004,
            regs: {R2: 1},
            memory: {0x3012: 1}
        }

        sequence! {
            ldr_zero,
            insns: [ { LEA R0, #-1}, {ADD R1, R1, #1}, {STR R1, R0, #0}, {LDR R2, R0, #0}],
            steps: Some(4),
            ending_pc: 0x3004,
            regs: {R2: 1},
            memory: {0x3000: 1}
        }

        sequence! {
            ldr_neg,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {STR R1, R0, #-1}, {LDR R2, R0, #-1}],
            steps: Some(4),
            ending_pc: 0x3004,
            regs: {R2: 1},
            memory: {0x3010: 1}
        }
    }

    /////////
    // LDI //
    /////////
    mod ldi {
        use super::*;

        sequence! {
            ldi_pos,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {ST R0, #3}, {STI R1, #2}, {LDI R2, #1}],
            steps: Some(5),
            ending_pc: 0x3005,
            regs: {R2: 1},
            memory: {0x3011: 1}
        }

        sequence! {
            ldi_zero,
            insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {ST R0, #2}, {STI R1, #1}, {LDI R2, #0}],
            steps: Some(5),
            ending_pc: 0x3005,
            regs: {R2: 1},
            memory: {0x3011: 1}
        }

        sequence! {
            ldi_neg,
            insns: [ { LEA R0, #-1}, {ADD R1, R1, #1}, {ST R0, #-1}, {STI R1, #-2}, {LDI R2, #-3}],
            steps: Some(5),
            ending_pc: 0x3005,
            regs: {R2: 1},
            memory: {0x3000: 1}
        }
    }

    /////////
    // LEA //
    /////////
    mod lea {
        use super::*;

        sequence! {
            lea_pos,
            insns: [ { LEA R0, #1} ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: {R0: 0x3002},
            memory: {}
        }

        sequence! {
            lea_zero,
            insns: [ { LEA R0, #0} ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: {R0: 0x3001},
            memory: {}
        }

        sequence! {
            lea_neg,
            insns: [ { LEA R0, #-1} ],
            steps: Some(1),
            ending_pc: 0x3001,
            regs: {R0: 0x3000},
            memory: {}
        }
    }

    //////////
    // TRAP //
    //////////
    mod trap {
        use super::*;

        sequence! {
            trap_0,
            insns: [ { ADD R6, R6, #15}, {TRAP #1} ],
            steps: Some(2),
            ending_pc: 0x0000,
            regs: {R6: 13},
            memory: {}
        }

        sequence! {
            trap_1,
            insns: [ {LEA R1, #-2}, { ADD R2, R2, #14}, {STR R1, R2, #0}, {ADD R6, R6, #14}, {TRAP #14} ],
            steps: Some(5),
            ending_pc: 0x2fff,
            regs: {R6: 12},
            memory: {}
        }
    }

    /////////
    // RTI //
    /////////
    mod rti {
        use super::*;

        sequence! {
            rti_0,
            // R1 <- x3001, R2 <- 10, xA <- R1, TRAP at xA, RTI
            //insns: [ {LEA R1, #-2}, { ADD R2, R2, #14}, {STR R1, R2, #0}, {ADD R6, R6, #14}, {TRAP #14} ],
            insns: [
                { BRnzp #2 },
                { ADD R5, R5, #15 },
                { RTI },
                { LEA R1, #-3 },
                { ADD R2, R2, #10 },
                { STR R1, R2, #0 },
                { ADD R6, R6, #14 },
                { TRAP #10 },
                { ADD R5, R5, #15 }
            ],
            steps: Some(9),
            ending_pc: 0x3009,
            regs: {R6: 14, R5: 30, R2: 10}, // R6 = 14 because it popped PC and PSR when RTI-ing
            memory: {}
        }
    }
}
