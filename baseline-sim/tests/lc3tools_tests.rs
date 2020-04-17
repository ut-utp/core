extern crate lc3_test_infrastructure as lti;

use lti::{insn, Addr, Instruction, Reg, Word};
use lti::{MemoryShim, PeripheralsShim};

use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;
use std::fs::{File, remove_file};
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;
use std::io::Write;
use lc3_baseline_sim::interp::{PeripheralInterruptFlags, InstructionInterpreter,
    Interpreter, InterpreterBuilder, MachineState
};
extern crate rand;
use rand::Rng;
use core::convert::{TryFrom, TryInto};

use lti::assert_eq;

// The bash script will not work on Windows.
#[cfg(all(test, target_family = "unix"))]
mod lc3tools {
    use super::*;
    use lti::with_larger_stack;

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
