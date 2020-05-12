//! A test runner that runs a program on the [lc3tools] simulator and the UTP
//! interpreter.
//!
//! [lc3tools]: https://github.com/chiragsakhuja/lc3tools

use crate::{
    Instruction, PeripheralInterruptFlags, Interpreter,
    InstructionInterpreter, Addr, Word, Reg,
};

use lc3_baseline_sim::interp::{MachineState, InterpreterBuilder};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;

use std::fs::{File, remove_file};
use std::io::{self, BufReader, Write};
use std::process::Command;
use std::convert::{TryFrom, TryInto};
use std::iter::once;
use std::env;

use rand::Rng;

#[derive(Debug)]
pub struct MemEntry {
    addr: Addr,
    word: Word,
}

// TODO: make this attempt to _install_ lc3tools? Not sure but we should do
// better than the below.

// TODO: ditch the bash script for sending things through pipes straight into
// a `Child`.

// TODO: we assume that there are no branches or control flow instructions! This
// is very limiting!

// TODO: actually check the final PC, PSR, CC, and MCR.

const DEBUG: bool = false;

macro_rules! d {
    ($e:expr) => {
        if DEBUG {
            dbg!($e)
        } else {
            $e
        }
    };
}

pub fn lc3tools_tester<'flags, M: Memory + Default + Clone, P: Peripherals<'flags>>
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
    let lc3_asm_file_path = &format!("{}/test_lc3_{}.txt    ", OUT_DIR, test_num);
    let mut lc3_asm_file = File::create(lc3_asm_file_path)?;

    // Write out the instructions:
    let iter = once(format!("{}", insns.len()))       // start with the number of instructions
        .chain(once(".orig x3000".to_string()))       // followed by the orig..
        .chain(insns.iter().map(ToString::to_string)) // ..the instructions..
        .chain(once(".end".to_string()));             // and finally, the end

    for line in iter {
        writeln!(lc3_asm_file, "{}", d!(line))?
    }

    // Run the program in lc3tools and collect the trace.
    let mut output = Command::new(format!("{}/lc3tools_executor.sh", OUT_DIR))
        .arg(lc3_asm_file_path)
        .arg(lc3tools_bin_dir)
        .output()?
        .stdout;

    let mut memory = Vec::<MemEntry>::new();
    let mut regs = [None; Reg::NUM_REGS];

    // TODO: actually check the final PC, PSR, CC, and MCR.
    let mut pc = None;
    let mut psr = None;
    let mut cc = None;
    let mut mcr = None;

    // for `0x25` style formatting
    fn parse_hex_val(v: &str) -> u16 {
        d!(v);
        Word::from_str_radix(&v[2..], 16).unwrap()
    }

    fn parse_reg(r: &str) -> Option<Word> {
        match r.split(" ").collect::<Vec<_>>().as_slice() {
            [_, r, ..] => Some(parse_hex_val(r)),
            _ => unreachable!(),
        }
    }

    let output = String::from_utf8(output).unwrap();
    if DEBUG { println!("{}", output) }

    for line in output.lines().filter(|l| !l.is_empty()) {
        match d!(line) {
            pair if line.starts_with("0x") => {
                match pair.split(": ").collect::<Vec<&str>>().as_slice() {
                    [addr, word, ..] => memory.push(MemEntry {
                        addr: parse_hex_val(addr),
                        word: parse_hex_val(&word[0..6]),
                    }),
                    _ => unreachable!(),
                }
            },

            pc_val if line.starts_with("PC") => {
                match pc_val.split(" ").collect::<Vec<_>>().as_slice() {
                    [_, pc_val, ..] => pc = Some(parse_hex_val(pc_val)),
                    _ => unreachable!(),
                }
            }

            psr_val if line.starts_with("PSR") => {
                match psr_val.split(" ").collect::<Vec<_>>().as_slice() {
                    [_, psr_val, ..] => psr = Some(parse_hex_val(psr_val)),
                    _ => unreachable!(),
                }
            }

            cc_val if line.starts_with("CC") => {
                match cc_val.split(" ").collect::<Vec<_>>().as_slice() {
                    [_, cc_val, ..] => cc = Some(cc_val),
                    _ => unreachable!(),
                }
            }

            mcr_val if line.starts_with("MCR") => {
                match mcr_val.split(" ").collect::<Vec<_>>().as_slice() {
                    [_, mcr_val, ..] => mcr = Some(parse_hex_val(mcr_val)),
                    _ => unreachable!(),
                }
            }

            lower_regs if line.contains("R0:") => {
                match lower_regs.split("R").collect::<Vec<_>>().as_slice() {
                    [_, r0, r1, r2, r3, ..] => [r0, r1, r2, r3].iter()
                        .map(|s| parse_reg(s))
                        .enumerate()
                        .for_each(|(idx, val)| regs[idx] = val),
                    _ => unreachable!(),
                }
            }

            upper_regs if line.contains("R4:") => {
                match upper_regs.split("R").collect::<Vec<_>>().as_slice() {
                    [_, r4, r5, r6, r7, ..] => [r4, r5, r6, r7].iter()
                        .map(|s| parse_reg(s))
                        .enumerate()
                        .for_each(|(idx, val)| regs[idx + 4] = val),
                    _ => unreachable!(),
                }
            }

            _ => {},
        }
    }

    // Run the program on the UTP simulator:
    let mut addr = 0x3000;

    let mut interp: Interpreter<'flags, M, P> = if let Some((mem, addr)) = alt_memory {
        let mut int: Interpreter<'flags, M, P> = InterpreterBuilder::new()
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

    // TODO: we assume that there are no branches or control flow instructions!
    // This is very limiting!
    let steps = insns.len();

    for insn in insns {
        interp.set_word_unchecked(addr, insn.into());
        addr += 1;
    }

    for _ in 0..steps {
        interp.step();
    }

     // Check registers:
     for (idx, reg) in regs.iter().enumerate() {
            let expected = reg.unwrap();
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
    for MemEntry { addr, word } in memory.iter() {
        let val = interp.get_word_unchecked(*addr);
        if *addr > 768 {
            assert_eq!(
                *word, val,
                "Expected memory location {:#04X} to be {:#04X} (lc3tools), was {:?}",
                addr, val, word
            );
        }
    }

    Ok(())
}
