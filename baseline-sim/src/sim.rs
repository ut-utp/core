use lc3_isa::{Addr, Bits, Instruction, Reg, Word};
use lc3_traits::control::Control;
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::{PeripheralSet, Peripherals};

use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use core::cell::Cell;

#[derive(Copy, Clone, Debug, PartialEq)]
enum SimulatorState {
    Running,
    Halted,
}

struct PeripheralState {
    input_character_ready: Cell<bool>,
}

struct Simulator<'a, M: Memory, P: Peripherals<'a>> {
    mem: M,
    peripherals: P,
    regs: [Word; 8],
    pc: Word,
    state: SimulatorState,
    peripheral_state: PeripheralState,
    _p: PhantomData<&'a ()>,

}

impl<'a, M: Memory, P: Peripherals<'a>> Default for Simulator<'a, M, P> {
    fn default() -> Self {
        unimplemented!()
    }
}

// TODO: use mem's get word on the Control impl

pub const KBSR: Addr = 0xFE00;
pub const KBDR: Addr = 0xFE02;
pub const DSR: Addr = 0xFE04;
pub const DDR: Addr = 0xFE06;
pub const BSP: Addr = 0xFFFA; // TODO: need to initialize this as 0x2FFE
pub const PSR: Addr = 0xFFFC;
pub const MCR: Addr = 0xFFFE;

// TODO: Try to implement the control trait for the simulator!
// TODO: Need to handle LC-3 exceptions and additional exceptions!

pub trait Sim {
    fn step(&mut self) -> Result<SimulatorState, ()>;

    fn set_pc(&mut self, addr: Addr);
    fn get_pc(&self) -> Addr;

    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt;
    fn get_word(&self, addr: Addr) -> ReadAttempt;

    fn get_register(&self, reg: Reg) -> Word;
    fn set_register(&mut self, reg: Reg, word: Word);
}

impl<'a, M: Memory, P: Peripherals<'a>> Index<Reg> for Simulator<'a, M, P> {
    type Output = Word;

    fn index(&self, reg: Reg) -> &Word {
        &self.regs[TryInto::<usize>::try_into(Into::<u8>::into(reg)).unwrap()]
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> IndexMut<Reg> for Simulator<'a, M, P> {
    fn index_mut(&mut self, reg: Reg) -> &mut Word {
        &mut self.regs[TryInto::<usize>::try_into(Into::<u8>::into(reg)).unwrap()]
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> Simulator<'a, M, P> {
    fn set_cc(&mut self, word: Word) {
        // n is the high bit:
        let n: bool = (word >> ((core::mem::size_of::<Word>() * 8) - 1)) != 0;

        // z is easy enough to check for:
        let z: bool = word == 0;

        // if we're not negative or zero, we're positive:
        let p: bool = !(n | z);

        fn bit_to_word(bit: bool, left_shift: u32) -> u16 {
            (if bit { 1 } else { 0 }) << left_shift
        }

        let b = bit_to_word;

        self.mem.write_word(
            PSR,
            (self.mem.read_word(PSR) & !(0x0007)) | b(n, 2) | b(z, 1) | b(p, 0),
        );
    }

    fn get_cc(&self) -> (bool, bool, bool) {
        let psr = self.mem.read_word(PSR);

        (psr.bit(2), psr.bit(1), psr.bit(0))
    }

    fn handle_exception(&mut self, ex_vec: u8) {
        let psr = self.mem.read_word(PSR);

        // If we're in user mode..
        if psr.bit(15) {
            // ..switch to supervisor mode..
            self.mem.write_word(PSR, psr | 0x8000);

            // ..and switch the stacks:
            let (usp, ssp) = (self[Reg::R6], self.mem.read_word(BSP));
            self.mem.write_word(BSP, usp);
            self[Reg::R6] = ssp;
        } else {
            // I'm assuming if PSR[15] is in supervisor mode, R6 is already the supervisor stack
            // pointer and BSR has the user stack pointer.
        }

        self[Reg::R6] -= 1;
        self.mem.write_word(self[Reg::R6], self.mem.read_word(PSR));
        self[Reg::R6] -= 1;
        self.mem.write_word(self[Reg::R6], self.pc);
        self.pc = 0x0100 | (ex_vec as u16);
    }

    fn handle_interrupt(&mut self, int_vec: u8, priority: u8) -> bool {
        let psr = self.mem.read_word(PSR);
        if psr.bit(15) {
            self.mem.write_word(PSR, psr | 0x8000);

            // ..and switch the stacks:
            let (usp, ssp) = (self[Reg::R6], self.mem.read_word(BSP));
            self.mem.write_word(BSP, usp);
            self[Reg::R6] = ssp;
        } else {
            // no swap!
        }

        // Push the PSR and _then_ the PC (so that we pop the PSR after the PC).
        self[Reg::R6] -= 1;
        self.mem.write_word(self[Reg::R6], self.mem.read_word(PSR));
        self[Reg::R6] -= 1;
        self.mem.write_word(self[Reg::R6], self.pc);

        if self.mem.read_word(PSR).u8(8..10) < priority {
            // Clear to interrupt
            self.pc = 0x0100 | (int_vec as u16);
            true
        } else {
            // Gotta wait
            false
        }

    }

    fn is_acv(&self, addr: Word) -> bool {
        let psr = self.mem.read_word(PSR);
        (addr < 0x3000) | (addr > 0xFE00) & psr.bit(15)
    }
}

enum ReadAttempt {
    Success(Word),
    Acv,
}

enum WriteAttempt {
    Success,
    Acv,
}

impl<'a, M: Memory, P: Peripherals<'a>> Sim for Simulator<'a, M, P> {
    fn step(&mut self) -> Result<SimulatorState, ()> {
        use Instruction::*;

        if let SimulatorState::Halted = self.state {
            return Err(());
        }

        // Increment PC (state 18):
        let current_pc = self.get_pc();
        self.set_pc(current_pc.wrapping_add(1)); // TODO: ???

        // Check for interrupts:
        // (in priority order)
        if self.peripheral_state.input_character_ready.get() {
            if self.mem.read_word(KBSR).bit(14) {
                if self.handle_interrupt(0x80, 4) {
                    self.peripheral_state.input_character_ready.set(false);
                }
            }
        }

        match self.get_word(current_pc) {
            ReadAttempt::Success(word) => match word.try_into() {
                Ok(ins) => match ins {
                    AddReg { dr, sr1, sr2 } => {
                        self[dr] = self[sr1].wrapping_add(self[sr2]);
                        self.set_cc(self[dr]);
                    }
                    AddImm { dr, sr1, imm5 } => {
                        self[dr] = self[sr1] + imm5 as Word;
                        self.set_cc(self[dr]);
                    }
                    AndReg { dr, sr1, sr2 } => {
                        self[dr] = self[sr1] & self[sr2];
                        self.set_cc(self[dr]);
                    }
                    AndImm { dr, sr1, imm5 } => {
                        self[dr] = self[sr1] & imm5 as Word;
                        self.set_cc(self[dr]);
                    }
                    Br { n, z, p, offset9 } => {
                        let (N, Z, P) = self.get_cc();
                        if n & N || z & Z || p & P {
                            self.set_pc(self.get_pc().wrapping_add(offset9 as Word))
                        }
                    }
                    Jmp { base } => {
                        self.set_pc(self[base]);
                    }
                    Jsr { offset11 } => {
                        self[Reg::R7] = self.get_pc();
                        self.set_pc(self.get_pc().wrapping_add(offset11 as Word));
                    }
                    Jsrr { base } => {
                        // TODO: add a test where base _is_ R7!!
                        let (pc, new_pc) = (self.get_pc(), self[base]);
                        self.set_pc(new_pc);
                        self[Reg::R7] = pc;
                    }
                    Ld { dr, offset9 } => {
                        // TODO: Need to check if address is KBSR or KBDR.
                        match self.get_word(self.get_pc().wrapping_add(offset9 as Word)) {
                            ReadAttempt::Success(word) => { self[dr] = word; self.set_cc(self[dr]); }
                            ReadAttempt::Acv => {}
                        }
                    }
                    Ldi { dr, offset9 } => {
                        match self.get_word(self.get_pc().wrapping_add(offset9 as Word)) {
                            ReadAttempt::Success(indir) => match self.get_word(indir) {
                                ReadAttempt::Success(word) => {
                                    self[dr] = word;
                                    self.set_cc(self[dr]);
                                }
                                ReadAttempt::Acv => {}
                            }
                            ReadAttempt::Acv => {}
                        }
                    }
                    Ldr { dr, base, offset6 } => {
                        match self.get_word(self[base].wrapping_add(offset6 as Word)) {
                            ReadAttempt::Success(word) => {
                                self[dr] = word;
                                self.set_cc(self[dr]);
                            }
                            ReadAttempt::Acv => {}
                        }
                    }
                    Lea { dr, offset9 } => {
                        self[dr] = self.get_pc().wrapping_add(offset9 as Word);
                    }
                    Not { dr, sr } => {
                        self[dr] = !self[sr];
                        self.set_cc(self[dr]);
                    }
                    Ret => {
                        self.set_pc(self[Reg::R7]);
                    }
                    Rti => {
                        let psr = self.mem.read_word(PSR);
                        if (psr.bit(15)) == false { // In supervisor mode
                            self.set_pc(self.mem.read_word(self[Reg::R6])); // TODO: make this an unwrap
                            self[Reg::R6] += 1;
                            self.mem.write_word(PSR, self.mem.read_word(self[Reg::R6])); // TODO: make this an unwrap
                            self[Reg::R6] += 1;

                            if self.mem.read_word(PSR).bit(15) { // If we're going back to user mode, swap the stack pointers
                                let (usp, ssp) = (self.mem.read_word(BSP), self[Reg::R6]);
                                self.mem.write_word(BSP, ssp);
                                self[Reg::R6] = usp;
                            }
                        } else {
                            self.handle_exception(0x00);
                        }
                    }
                    St { sr, offset9 } => {
                        match self.set_word(self.get_pc().wrapping_add(offset9 as Word), self[sr]) {
                            WriteAttempt::Success => {}
                            WriteAttempt::Acv => {}
                        }
                    }
                    Sti { sr, offset9 } => {
                        match self.get_word(self.get_pc().wrapping_add(offset9 as Word)) {
                            ReadAttempt::Success(indir) => {
                                match self.set_word(indir, self[sr]) {
                                    WriteAttempt::Success => {}
                                    WriteAttempt::Acv => {}                                }
                            }
                            ReadAttempt::Acv => {}
                        }
                    }
                    Str { sr, base, offset6 } => {
                        match self.set_word(self[base].wrapping_add(offset6 as Word), self[sr]) {
                            WriteAttempt::Success => {},
                            WriteAttempt::Acv => {},
                        }
                    }
                    Trap { trapvec } => {
                        if self.mem.read_word(PSR).bit(15) { // User mode going into supervisor mode
                            let (usp, ssp) = (self[Reg::R6], self.mem.read_word(BSP));
                            self.mem.write_word(BSP, usp);
                            self[Reg::R6] = ssp;

                            self.mem.write_word(PSR, self.mem.read_word(PSR) & !(0x8000));
                        }

                        self[Reg::R6] -= 1;
                        self.mem.write_word(self[Reg::R6], self.mem.read_word(PSR));
                        self[Reg::R6] -= 1;
                        self.mem.write_word(self[Reg::R6], self.pc);

                        self.set_pc(self.mem.read_word(trapvec as u16));
                    }
                },
                Err(_) => {
                    self.handle_exception(0x01);
                }
            },
            ReadAttempt::Acv => {}
        }

        Ok(self.state)
    }

    fn set_pc(&mut self, addr: Addr) {
        self.pc = addr;
    }
    fn get_pc(&self) -> Addr {
        self.pc
    }

    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt {
        if self.is_acv(addr) {
            self.handle_exception(0x02);
            return WriteAttempt::Acv;
        }

        match addr {
            KBDR => {},
            MCR => {
                if !word.bit(15) {
                    // Halt!
                    self.state = SimulatorState::Halted;
                }
                self.mem.write_word(addr, word)
            }
            _ => self.mem.write_word(addr, word)

        }

        WriteAttempt::Success
    }

    fn get_word(&self, addr: Addr) -> ReadAttempt {
        if self.is_acv(addr) {
            self.handle_exception(0x02);
            return ReadAttempt::Acv;
        }

        ReadAttempt::Success(match addr {
            KBDR
            _ => self.mem.read_word(addr)
        })
    }

    fn get_register(&self, reg: Reg) -> Word {
        self[reg]
    }

    fn set_register(&mut self, reg: Reg, word: Word) {
        self[reg] = word;
    }

    // fn get_state(&self) -> State {
    //     self.state
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::isa::Instruction;

//     // Test that the instructions work
//     // Test that the unimplemented instructions do <something>

//     fn interp_test_runner<'a, M: Memory, P: Peripherals<'a>>(
//         insns: Vec<Instruction>,
//         num_steps: Option<usize>,
//         regs: [Option<Word>; 8],
//         pc: Addr,
//         memory_locations: Vec<(Addr, Word)>,
//     ) {
//         let mut interp = Simulator::<M, P>::default();

//         let mut addr = 0x3000;
//         for insn in insns {
//             interp.set_word(addr, insn.into());
//             addr += 2;
//         }

//         if let Some(num_steps) = num_steps {
//             for _ in 0..num_steps {
//                 interp.step();
//             }
//         } else {
//             // TODO! (run until halted)
//         }

//         for (idx, r) in regs.iter().enumerate() {
//             if let Some(reg_word) = r {
//                 assert_eq!(interp.get_register(idx.into()), *reg_word);
//             }
//         }
//     }

//     #[test]
//     //NO-OP do nothing run a cycle test
//     fn nop() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Br {
//                 n: true,
//                 z: true,
//                 p: true,
//                 offset11: -1,
//             }],
//             Same(1),
//             [None, None, None, None, None, None, None, None],
//             0x3000,
//             vec![],
//         )
//     }
//     //0+1=1 Basic Add
//     #[test]
//     fn add_reg_test() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 1,
//                 },
//                 AddReg {
//                     dr: 2,
//                     sr1: 1,
//                     sr2: 0,
//                 },
//             ],
//             Some(1),
//             [Some(0), Some(1), Some(1), None, None, None, None, None],
//             0x3001,
//             vec![],
//         )
//     }
//     //AddImm Test with R0(0) + !
//     #[test]
//     fn AddImmTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::AddImm {
//                 dr: R0,
//                 sr1: R0,
//                 imm5: 1,
//             }],
//             Some(1),
//             [1, None, None, None, None, None, None, None],
//             0x3001,
//             vec![],
//         )
//     }
//     //AndReg Test with R0(1) and R1(2) to R0(expected 3)
//     #[test]
//     fn AndRegTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 AddImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 2,
//                 },
//                 AndReg {
//                     dr: R0,
//                     sr1: R0,
//                     sr2: R1,
//                 },
//             ],
//             Some(3),
//             [3, 2, None, None, None, None, None, None],
//             0x3003,
//             vec![],
//         )
//     }
//     //AndImm Test with R1 (1) and 0
//     #[test]
//     fn AndImmTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 1,
//                 },
//                 AndImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 0,
//                 },
//             ],
//             Some(2),
//             [0, None, None, None, None, None, None, None],
//             0x3002,
//             vec![],
//         )
//     }
//     //ST Test which stores 1 into x3001
//     #[test]
//     fn StTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 St { sr: R0, offset9: 0 },
//             ],
//             Some(2),
//             [1, None, None, None, None, None, None, None],
//             0x3002,
//             vec![(0x3001, 1)],
//         )
//     }
//     //LD Test with R0 and memory
//     #[test]
//     fn LdTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 St { sr: R0, offset9: 1 },
//                 Ld { dr: R0, offset9: 0 },
//             ],
//             Some(3),
//             [3001, None, None, None, None, None, None, None],
//             0x3003,
//             vec![(0x3001, 1)],
//         )
//     }
//     //LDR Test with R0 and memory
//     #[test]
//     fn LdrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 St { sr: R0, offset9: 0 },
//                 Ldr {
//                     dr: R1,
//                     offset9: -1,
//                 },
//             ],
//             Some(3),
//             [1, 3001, None, None, None, None, None, None],
//             0x3003,
//             vec![(0x3001, 1)],
//         )
//     }
//     //Load x3000 into R1
//     #[test]
//     fn LeaTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R0, offset9: 0 }],
//             Some(1),
//             [3000, None, None, None, None, None, None, None],
//             0x3001,
//             vec![],
//         )
//     }
//     // STR test with offset store into lea using 3000
//     #[test]
//     fn StrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R1, offset9: 0 },
//                 Lea { dr: R2, offset9: 1 },
//                 Str {
//                     sr: R2,
//                     base: R1,
//                     offset6: 1,
//                 },
//             ],
//             Some(3),
//             [None, None, None, None, None, None, None, None],
//             0x3003,
//             vec![(x3004, 3000)],
//         )
//     }
//     //not test
//     #[test]
//     fn NotTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 Not { dr: R1, sr: R0 },
//             ],
//             Some(2),
//             [1, 0, None, None, None, None, None, None],
//             0x3002,
//             vec![],
//         )
//     }
//     //ldi Test using location 3000 and loading value of memory into register, using 3002 and 3001 holding 3000 as reference
//     #[test]
//     fn LdiTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R0, offset9: 0 },
//                 St { sr: R0, offset9: 0 },
//                 St {
//                     sr: R0,
//                     offset9: -2,
//                 },
//                 Ldi {
//                     dr: R2,
//                     offset9: -1,
//                 },
//             ],
//             Some(4),
//             [1, None, 3000, None, None, None, None, None],
//             0x3004,
//             vec![(x3001, 3000), (x3000, 3000)],
//         )
//     }
//     //jumps to R7 register, loaded with memory address 3005
//     #[test]
//     fn RetTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R7, offset9: 5 }, Ret],
//             Some(2),
//             [None, None, None, None, None, None, None, 3005],
//             0x3005,
//             vec![],
//         )
//     }
//     //STI test, stores 3000 in register 1 and sets that to the memory at x3002 so sti writes to memory location 3000
//     #[test]
//     fn StiTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R0, offset9: 0 },
//                 St { sr: R0, offset6: 2 },
//                 AddImm {
//                     dr: R3,
//                     sr1: R3,
//                     imm5: 1,
//                 },
//                 Sti { sr: R3, offset9: 0 },
//             ],
//             Some(4),
//             [3000, None, None, 1, None, None, None, None],
//             0x3004,
//             vec![(x3003, 3000), (x3000, 1)],
//         )
//     }
//     //Jump Test, switch PC to value in register
//     #[test]
//     fn JmpTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R0, offset9: 0 }, Jmp { base: R0 }],
//             Some(2),
//             [3000, None, None, None, None, None, None, None],
//             0x3000,
//             vec![],
//         )
//     }
//     //jsrr test, jumps to location 3005 and stores 3001 in r7
//     #[test]
//     fn JsrrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R0, offset9: 5 }, Jsrr { base: R0 }],
//             Some(2),
//             [3000, None, None, None, None, None, None, 3001],
//             0x3005,
//             vec![],
//         )
//     }
//     //jsr test, jumps back to queue location from r7
//     #[test]
//     fn JsrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R0, offset9: 5 },
//                 St { sr: R0, offset6: 2 },
//                 Jsr { offset11: 1 },
//             ],
//             Some(3),
//             [3000, None, None, None, None, None, None, 3001],
//             0x3000,
//             vec![],
//         )
//     }
// }
