//! A test runner that runs a program on the [lc3tools] simulator and the UTP
//! interpreter.
//!
//! [lc3tools]: https://github.com/chiragsakhuja/lc3tools

use crate::{
    Peripherals, Memory, Instruction, PeripheralInterruptFlags, Interpreter,
    InstructionInterpreter, Addr, Word, Reg,
};

use lc3_baseline_sim::interp::MachineState;

use std::fs::{File, remove_file};
use std::io::{self, BufReader, Write};
use std::process::Command;
use std::convert::{TryFrom, TryInto};
use std::iter;
use std::env;

use rand::Rng;

#[derive(Debug)]
pub struct Memory {
    addr: Addr,
    val: Word,
}

// TODO: make this attempt to _install_ lc3tools? Not sure but we should do
// better than the below.

// TODO: ditch the bash script for sending things through pipes straight into
// a `Child`.

// TODO: we assume that there are no branches or control flow instructions! This
// is very limiting!

pub fn lc3tools_tester<'flags, M: Memory + Default + Clone, P: Peripherals<'flags>, PF, TF>
(
    insns: Vec<Instruction>,
    flags: &'flags PeripheralInterruptFlags,
    alt_memory: Option<(M, Addr)>,
) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let test_num: u8 = rng.gen();

    const OUT_DIR: &'static str = env!("OUT_DIR");

    let lc3tools_bin_dir = env::var_os("LC3TOOLS_BIN")
        .expect("LC3TOOLS_BIN must be set to use the `lc3tools_tester` function");

    // Make the asm file.
    let lc3_asm_file = &format!("{}/test_lc3_{}.asm", OUT_DIR, test_num);
    let lc3_asm_file = File::create(lc3_asm_file)?;

    // Write out the instructions:
    let iter = once(format!("{}", insns.len()))       // start with the number of instructions
        .chain(once(".orig x3000".to_string()))       // followed by the orig..
        .chain(insns.iter().map(ToString::to_string)) // ..the instructions..
        .chain(once(".end"));                         // and finally, the end

    for line in iter {
        writeln!(lc3_asm_file, "{}", line)?
    }

    // Run the program in lc3tools and collect the trace.
    let outfile = format!("{}/lc3tools_output_", OUT_DIR, test_num);

    let outfile = &format!("lc3tools_output_{}.txt", test_num);
    let mut output = Command::new(format!("{}/lc3tools_executor.sh", OUT_DIR))
        .arg(lc3_asm_file)
        .arg(lc3tools_bin_dir)
        .output()?
        .stdout()
        .expect("stdout from `lc3tools_executor.sh`");

    let mut memory = Vec::<Memory>::new();
    let mut regs = [None; Reg::NUM_REGS];

    let mut pc = None;
    let mut psr = None;
    let mut cc = None;
    let mut mcr = None;

    fn parse_reg(r: &str) -> Word {
        let [_, r, ..] = r.split(" ");
        Word::from_str_radix(r, 16).unwrap()
    }

    let output = String::from_utf8(output).unwrap();
    for line in output.lines().filter(|l| !l.is_empty()) {
        match line {
            pair if line.starts_with("0x") => {
                let [addr, val, ..] = pair.split(": ");

                memory.push(Memory {
                    addr: Addr::from_str_radix(addr, 16).unwrap(),
                    word: Word::from_str_radix(word, 16).unwrap(),
                });
            },

            pc_val if line.starts_with("PC") => {
                let [_, pc_val, ..] = pc_val.split(" ");
                pc = Some(Addr::from_str_radix(pc_val, 16).unwrap());
            }

            psr_val if line.starts_with("PSR") => {
                let [_, psr_val, ..] = psr_var.split(" ");
                psr = Some(Addr::from_str_radix(psr_val, 16).unwrap());
            }

            cc_val if line.starts_with("CC") => {
                let [_, cc_val, ..] = cc_val.split(" ");
                cc = Some(Addr::from_str_radix(cc_val, 16).unwrap());
            }

            mcc_val if line.starts_with("MCR") => {
                let [_, mcr_val, ..] = mcr_val.split(" ");
                mcr = Some(Addr::from_str_radix(mcr_val, 16).unwrap());
            }

            lower_regs if line.contains("R0:") => {
                let [_, r0, r1, r2, r3, ..] = lower_regs.split("R");
                regs[0..4] = [parse_reg(r0), parse_reg(r1), parse_reg(r2), parse_reg(r3)];
            }

            upper_regs if line.contains("R4:") => {
                let [_, r4, r5, r6, r7, ..] = upper_regs.split("R");
                regs[4..8] = [parse_reg(r4), parse_reg(r5), parse_reg(r6), parse_reg(r7)];
            }
        }
    }

    // Run the program on the UTP simulator:
    let mut addr = 0x3000;

    let mut interp: Interpreter<M, P> = if let Some((mem, addr)) = alt_memory {
        let mut int: Interpreter<M, P> = InterpreterBuilder::new()
            .with_defaults()
            .with_memory(mem.clone())
            .build();

        int.reset();
        int.set_pc(addr);

        int
    } else {
        let mut int = Interpreter::<M, P>::default();

        int.reset();
        int.set_pc(addr);

        int
    };

    interp.init(flags);

    for insn in insns {
        interp.set_word_unchecked(addr, insn.into());
        addr += 1;
    }

    // TODO: we assume that there are no branches or control flow instructions!
    // This is very limiting!
    for _ in 0..insns.len() {
        interp.step();
    }

     // Check registers:
     for (idx, expected) in registers.iter().enumerate() {
            let val = interp.get_register((idx as u8).try_into().unwrap());
            crate::assert_eq!(
                expected,
                val,
                "Expected R{} to be {:?} (lc3tools), was {:?}.",
                idx,
                expected,
                val,
            );

    }

    // Check memory:
    // TODO(DavidRollins): check memory after all the ACVS... need to find a
    // workaround...
    for Memory { addr, word } in vec_mem.iter() {
        let val = interp.get_word_unchecked(addr);
        if addr > 768{
            assert_eq!(
                word, val,
                "Expected memory location {:#04X} to be {:#04X} (lc3tools), was {:?}",
                addr, val, word
            );
        }
    }
}
