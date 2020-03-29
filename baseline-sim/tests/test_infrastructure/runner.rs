use lc3_isa::{Addr, Instruction, Word};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;

use lc3_baseline_sim::interp::{PeripheralInterruptFlags, InstructionInterpreter, Interpreter, InterpreterBuilder, MachineState};

use core::convert::{TryFrom, TryInto};

use pretty_assertions::assert_eq;

pub fn interp_test_runner<'a, M: Memory + Default + Clone, P: Peripherals<'a>, PF, TF>
(
    prefilled_memory_locations: Vec<(Addr, Word)>,
    insns: Vec<Instruction>,
    num_steps: Option<usize>,
    regs: [Option<Word>; 8],
    pc: Option<Addr>,
    memory_locations: Vec<(Addr, Word)>,
    setup_func: PF,
    teardown_func: TF,
    flags: &'a PeripheralInterruptFlags,
)
where
    for<'p> PF: FnOnce(&'p mut P),
    for<'p> TF: FnOnce(&'p P),
{
    let mut addr = 0x3000;

    interp.init(flags);

    // Run the setup func:
    setup_func(&mut *interp);

    // Prefill the memory locations:
    for (addr, word) in prefilled_memory_locations.iter() {
        // Crashes on ACVs! (they should not happen at this point)
        interp.set_word(*addr, *word).unwrap()
    }

    for insn in insns {
        // let enc = Into::<u16>::into(insn);
        // println!("{:?}", insn);
        // println!("{:#04X} -> {:?}", enc, Instruction::try_from(enc));
        interp.set_word_unchecked(addr, insn.into());
        // println!(
        //     "{:?}",
        //     Instruction::try_from(interp.get_word_unchecked(addr))
        // );

        addr += 1;
    }

    if let Some(num_steps) = num_steps {
        for _ in 0..num_steps {
            // println!("step: x{0:4X}", interp.get_pc());
            interp.step();
        }
    } else {
        while let MachineState::Running = interp.step() { }
    }

    // Check PC:
    if let Some(expected_pc) = pc {
        let actual_pc = interp.get_pc();
        assert_eq!(
            expected_pc,
            actual_pc,
            "Expected PC = {:#04X}, got {:#04X}",
            expected_pc,
            actual_pc
        );
    }


    // Check registers:
    for (idx, r) in regs.iter().enumerate() {
        if let Some(reg_word) = r {
            let val = interp.get_register((idx as u8).try_into().unwrap());
            assert_eq!(
                *reg_word,
                val,
                "Expected R{} to be {:?}, was {:?}",
                idx,
                *reg_word,
                val,
            );
        }
    }

    // Check memory:
    for (addr, word) in memory_locations.iter() {
        let val = interp.get_word_unchecked(*addr);
        assert_eq!(
            *word, val,
            "Expected memory location {:#04X} to be {:#04X}",
            *word, val
        );
    }

    // Run the teardown func:
    teardown_func(&*interp);
}
