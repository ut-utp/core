use lc3_baseline_sim::*;
use lc3_isa::{insn, Addr, Instruction, Reg, Word};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;

use lc3_baseline_sim::interp::{InstructionInterpreter, Interpreter, MachineState};

use lc3_shims::memory::MemoryShim;
use lc3_shims::peripherals::PeripheralsShim;

#[cfg(test)]
mod tests {
    use super::*;

    use Reg::*;

    use std::convert::TryInto;
    use std::convert::TryFrom;
    use lc3_isa::Reg::R0;


    // Test that the instructions work
    // Test that the unimplemented instructions do <something>

    fn interp_test_runner<'a, M: Memory + Default, P: Peripherals<'a>>(
        insns: Vec<Instruction>,
        num_steps: Option<usize>,
        regs: [Option<Word>; 8],
        pc: Addr,
        memory_locations: Vec<(Addr, Word)>,
    )
    // where for<'p> P: Peripherals<'p>
    {
        let mut interp = Interpreter::<M, P>::default();

        let mut addr = 0x3000;
        interp.reset();
        interp.set_pc(addr);

        for insn in insns {
            let enc = Into::<u16>::into(insn);
            println!("{:?}", insn);
            println!("{:#04X} -> {:?}", enc, Instruction::try_from(enc));
            interp.set_word_unchecked(addr, insn.into());
            println!("{:?}", Instruction::try_from(interp.get_word_unchecked(addr)));

            addr += 1;
        }

        if let Some(num_steps) = num_steps {
            for _ in 0..num_steps {
                // println!("step: x{0:4X}", interp.get_pc());
                interp.step();
            }
        } else {
            while let MachineState::Running = interp.get_machine_state() {
                interp.step();
            }
        }

        // Check PC:
        let expected_pc = pc;
        let actual_pc = interp.get_pc();
        assert_eq!(pc, interp.get_pc(), "Expected PC = {:#04X}, got {:#04X}", expected_pc, actual_pc);

        // Check registers:
        for (idx, r) in regs.iter().enumerate() {
            if let Some(reg_word) = r {
                assert_eq!(
                    interp.get_register((idx as u8).try_into().unwrap()),
                    *reg_word
                );
            }
        }

        // Check memory:
        for (addr, word) in memory_locations.iter() {
            assert_eq!(interp.get_word_unchecked(*addr), *word);
        }
    }

    macro_rules! sequence {
        ($(|$panics:literal|)? $name:ident, insns: [ $({ $($insn:tt)* }),* ], steps: $steps:expr, ending_pc: $pc:literal, regs: { $($r:tt: $v:expr),* }, memory: { $($addr:literal: $val:expr),* }) => {
        $(#[doc = $panics] #[should_panic])?
        #[test]
        fn $name() {

            #[allow(unused_mut)]
            let mut regs: [Option<Word>; Reg::NUM_REGS] = [None, None, None, None, None, None, None, None];
            $(regs[Into::<u8>::into($r) as usize] = Some($v);)*

            #[allow(unused_mut)]
            let mut checks: Vec<(Addr, Word)> = Vec::new();
            $(checks.push(($addr, $val));)*

            #[allow(unused_mut)]
            let mut insns: Vec<Instruction> = Vec::new();
            $(insns.push(insn!($($insn)*));)*

            interp_test_runner::<MemoryShim, PeripheralsShim>(
                insns,
                $steps,
                regs,
                $pc,
                checks
            );
        }};
    }

    // TODO: test macro like above but takes a program instead of a sequence of instructions (and uses the loadable! macro or the program macro).
    /////////
    // ADD //
    /////////
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
        regs: { R0: -1i16 as Word },
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

    /////////
    // AND //
    /////////
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

    ////////
    // BR //
    ////////
    sequence! {
        branch_self,
        insns: [ { BRnzp #-1 } ],
        steps: Some(1),
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

    /////////
    // JMP //
    /////////
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

    /////////
    // JSR //
    /////////
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

    //////////
    // JSRR //
    //////////
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
        insns: [ { ADD R0, R0, #1 }, { JSRR R0 }],
        steps: Some(2),
        ending_pc: 0x0001,
        regs: { R7: 0x3002 },
        memory: {}
    }


    ////////
    // LD //
    ////////
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

    /////////
    // NOT //
    /////////
    sequence! { // take 0
        not_0,
        insns: [ { ADD R0, R0, #0 }, { NOT R0, R0} ],
        steps: Some(2),
        ending_pc: 0x3002,
        regs: {R0: -1i16 as Word},
        memory: {}
    }
    
    sequence! {  // take a positive number
        not_1,
        insns: [ { ADD R0, R0, #1 }, { NOT R0, R0 } ],
        steps: Some(2),
        ending_pc: 0x3002,
        regs: {R0: -2i16 as Word},
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
    

    
    ////////
    // ST //
    ////////
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
        regs: {R0: -1i16 as Word},
        memory: {0x3012: -1i16 as Word}
    }

   
    sequence! { // store behind
        st_neg_offset,
        insns: [ { ADD R0, R0, #1}, {ST R0, #-16}],
        steps: Some(2),
        ending_pc: 0x3002,
        regs: {R0: 1},
        memory: {0x2FF2: 1}
    }

    /////////
    // RET //
    /////////
    sequence! { 
        ret_2,
        insns: [ { JSR #2 }, {ADD R0, R0, #0}, {ADD R0, R0, #0}, { RET } ],
        steps: Some(2),
        ending_pc: 0x3001,
        regs: { R7: 0x3001 },
        memory: {}
    }

    sequence! { 
        res_pos_neg,
        insns: [ { JSR #1 }, {RET}, {ADD R0, R0, #0}, {ADD R0, R0, #0}, { JSR #-4 } ],
        steps: Some(5),
        ending_pc: 0x3005,
        regs: { R7: 0x3005 },
        memory: {}
    }
    
    // load the return into a register -> store it somewhere -> jump there
    sequence! { 
        res_neg_pos,
        insns: [{LD R0, #3}, {ST R0, #-2}, {ADD R0, R0, #0}, { JSR #-4 }, {RET},  { JSR #-4 } ],
        steps: Some(5),
        ending_pc: 0x3004,
        regs: { R7: 0x3004 },
        memory: {}
    }
    // not sure how to test negative returns... 
    // Need to store return in a previous address
    // then can jump to there... ? 

    /////////
    // STI //
    /////////
    sequence! { 
        sti_pos,
        insns: [ { LEA R0, #16}, {ADD R1, R1, #1}, {ST R0, #2}, {STI R1, #1}],
        steps: Some(4),
        ending_pc: 0x3004,
        regs: {},
        memory: {0x3011: 1}
    }

    sequence! { 
        sti_neg,
        insns: [ { LEA R0, #-1}, {ADD R1, R1, #1}, {ST R0, #2}, {STI R1, #1}],
        steps: Some(4),
        ending_pc: 0x3004,
        regs: {},
        memory: {0x3000: 1}
    }


     /////////
    // STR //
    /////////
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

    // sequence! {
    //     ret_neg,
    //     insns: [ { JSR #-2 }, { RET } ],
    //     steps: Some(2),
    //     ending_pc: 0x3001,
    //     regs: { R7: 0x3001 },
    //     memory: {}
    // }

    // we can't write to values that are actually big enough to write to...
    

    // sequence! { 
    //     sti_1,
    //     insns: [ { ADD R1, R1, #1}, {ADD R0, R0, #1}, {ST R0, #16}, {STI R0, #15}],
    //     steps: Some(4),
    //     ending_pc: 0x3004,
    //     regs: {R0: 1},
    //     memory: {0x3012: 1, 0x0001: 1} 
    // }

    // sequence! { 
    //     sti_neg,
    //     insns: [ { ADD R0, R0, #-1}, {ST R0, #16}, {STI R0, #15}],
    //     steps: Some(3),
    //     ending_pc: 0x3003,
    //     regs: {R0: -1i16 as Word},
    //     memory: {0x3012: 1, 0x0001: 1} 
    // }    

    /////////
    // LDI //
    /////////
    // Hard without .FILL

    /////////
    // LDR //
    /////////
    // Hard without .FILL
    
    // #[test]
    // #[should_panic]
    // fn no_op_fail() {
    //     single! {
    //         insn: { BRnzp #2 }
    //         steps: Some(1),
    //         ending_pc: 0x3000,
    //         regs: {},
    //         memory: {}
    //     }
    //     // interp_test_runner::<MemoryShim, PeripheralsShim<'a>>(
    //     //     // vec![Instruction::Br {
    //     //     //     n: true,
    //     //     //     z: true,
    //     //     //     p: true,
    //     //     //     offset9: 67,
    //     //     // }],
    //     //     vec![insn!(BRnzp #67)],
    //     //     Some(1),
    //     //     [None, None, None, None, None, None, None, None],
    //     //     0x3000,
    //     //     vec![],
    //     // )
    // }
    // //0+1=1 Basic Add
    // #[test]
    // fn add_reg_test() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 1,
    //             },
    //             AddReg {
    //                 dr: 2,
    //                 sr1: 1,
    //                 sr2: 0,
    //             },
    //         ],
    //         Some(1),
    //         [Some(0), Some(1), Some(1), None, None, None, None, None],
    //         0x3001,
    //         vec![],
    //     )
    // }
    // //AddImm Test with R0(0) + !
    // #[test]
    // fn AddImmTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::AddImm {
    //             dr: R0,
    //             sr1: R0,
    //             imm5: 1,
    //         }],
    //         Some(1),
    //         [1, None, None, None, None, None, None, None],
    //         0x3001,
    //         vec![],
    //     )
    // }
    // //AndReg Test with R0(1) and R1(2) to R0(expected 3)
    // #[test]
    // fn AndRegTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             AddImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 2,
    //             },
    //             AndReg {
    //                 dr: R0,
    //                 sr1: R0,
    //                 sr2: R1,
    //             },
    //         ],
    //         Some(3),
    //         [3, 2, None, None, None, None, None, None],
    //         0x3003,
    //         vec![],
    //     )
    // }
    // //AndImm Test with R1 (1) and 0
    // #[test]
    // fn AndImmTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 1,
    //             },
    //             AndImm {
    //                 dr: R1,
    //                 sr1: R1,
    //                 imm5: 0,
    //             },
    //         ],
    //         Some(2),
    //         [0, None, None, None, None, None, None, None],
    //         0x3002,
    //         vec![],
    //     )
    // }
    // //ST Test which stores 1 into x3001
    // #[test]
    // fn StTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             St { sr: R0, offset9: 0 },
    //         ],
    //         Some(2),
    //         [1, None, None, None, None, None, None, None],
    //         0x3002,
    //         vec![(0x3001, 1)],
    //     )
    // }
    // //LD Test with R0 and memory
    // #[test]
    // fn LdTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             St { sr: R0, offset9: 1 },
    //             Ld { dr: R0, offset9: 0 },
    //         ],
    //         Some(3),
    //         [3001, None, None, None, None, None, None, None],
    //         0x3003,
    //         vec![(0x3001, 1)],
    //     )
    // }
    // //LDR Test with R0 and memory
    // #[test]
    // fn LdrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             St { sr: R0, offset9: 0 },
    //             Ldr {
    //                 dr: R1,
    //                 offset9: -1,
    //             },
    //         ],
    //         Some(3),
    //         [1, 3001, None, None, None, None, None, None],
    //         0x3003,
    //         vec![(0x3001, 1)],
    //     )
    // }
    // //Load x3000 into R1
    // #[test]
    // fn LeaTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R0, offset9: 0 }],
    //         Some(1),
    //         [3000, None, None, None, None, None, None, None],
    //         0x3001,
    //         vec![],
    //     )
    // }
    // // STR test with offset store into lea using 3000
    // #[test]
    // fn StrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R1, offset9: 0 },
    //             Lea { dr: R2, offset9: 1 },
    //             Str {
    //                 sr: R2,
    //                 base: R1,
    //                 offset6: 1,
    //             },
    //         ],
    //         Some(3),
    //         [None, None, None, None, None, None, None, None],
    //         0x3003,
    //         vec![(x3004, 3000)],
    //     )
    // }
    // //not test
    // #[test]
    // fn NotTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::AddImm {
    //                 dr: R0,
    //                 sr1: R0,
    //                 imm5: 1,
    //             },
    //             Not { dr: R1, sr: R0 },
    //         ],
    //         Some(2),
    //         [1, 0, None, None, None, None, None, None],
    //         0x3002,
    //         vec![],
    //     )
    // }
    // //ldi Test using location 3000 and loading value of memory into register, using 3002 and 3001 holding 3000 as reference
    // #[test]
    // fn LdiTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R0, offset9: 0 },
    //             St { sr: R0, offset9: 0 },
    //             St {
    //                 sr: R0,
    //                 offset9: -2,
    //             },
    //             Ldi {
    //                 dr: R2,
    //                 offset9: -1,
    //             },
    //         ],
    //         Some(4),
    //         [1, None, 3000, None, None, None, None, None],
    //         0x3004,
    //         vec![(x3001, 3000), (x3000, 3000)],
    //     )
    // }
    // //jumps to R7 register, loaded with memory address 3005
    // #[test]
    // fn RetTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R7, offset9: 5 }, Ret],
    //         Some(2),
    //         [None, None, None, None, None, None, None, 3005],
    //         0x3005,
    //         vec![],
    //     )
    // }
    // //STI test, stores 3000 in register 1 and sets that to the memory at x3002 so sti writes to memory location 3000
    // #[test]
    // fn StiTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R0, offset9: 0 },
    //             St { sr: R0, offset6: 2 },
    //             AddImm {
    //                 dr: R3,
    //                 sr1: R3,
    //                 imm5: 1,
    //             },
    //             Sti { sr: R3, offset9: 0 },
    //         ],
    //         Some(4),
    //         [3000, None, None, 1, None, None, None, None],
    //         0x3004,
    //         vec![(x3003, 3000), (x3000, 1)],
    //     )
    // }
    // //jsrr test, jumps to location 3005 and stores 3001 in r7
    // #[test]
    // fn JsrrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![Instruction::Lea { dr: R0, offset9: 5 }, Jsrr { base: R0 }],
    //         Some(2),
    //         [3000, None, None, None, None, None, None, 3001],
    //         0x3005,
    //         vec![],
    //     )
    // }
    // //jsr test, jumps back to queue location from r7
    // #[test]
    // fn JsrTest() {
    //     interp_test_runner::<MemoryShim, _>(
    //         vec![
    //             Instruction::Lea { dr: R0, offset9: 5 },
    //             St { sr: R0, offset6: 2 },
    //             Jsr { offset11: 1 },
    //         ],
    //         Some(3),
    //         [3000, None, None, None, None, None, None, 3001],
    //         0x3000,
    //         vec![],
    //     )
    // }
}
