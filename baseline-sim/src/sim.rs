use lc3_isa::{Addr, Bits, Instruction, Reg, Word};
use lc3_traits::control::Control;
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::{PeripheralSet, Peripherals};

use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

struct Simulator<'a, M: Memory, P: Peripherals<'a>> {
    mem: M,
    peripherals: P,
    regs: [Word; 8],
    pc: Word,
    _p: PhantomData<&'a ()>,
}

impl<'a, M: Memory, P: Peripherals<'a>> Default for Simulator<'a, M, P> {
    fn default() -> Self {
        unimplemented!()
    }
}

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
    fn step(&mut self);

    fn set_pc(&mut self, addr: Addr);
    fn get_pc(&self) -> Addr;

    fn set_word(&mut self, addr: Addr, word: Word);
    fn get_word(&self, addr: Addr) -> Word;

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
        if psr.bit(15) {
            self.mem.write_word(PSR, psr | 0x8000);
        } else {
            // I'm assuming if PSR[15] is in supervisor mode, R6 is already the supervisor stack
            // pointer and BSR has the user stack pointer.
            self[Reg::R6] = self.mem.read_word(BSP);
        }
        self[Reg::R6] -= 1;
        self.mem.write_word(self[Reg::R6], self.pc);
        self[Reg::R6] -= 1;
        self.mem.write_word(self[Reg::R6], psr);
        self.pc = 0x0100 | (ex_vec as u16);
    }

    fn set_acv(&self, addr: Word) {
        let psr = self.mem.read_word(PSR);
        let acv = (addr < 0x3000) | (addr > 0xFE00) & psr.bit(15);
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> Sim for Simulator<'a, M, P> {
    fn step(&mut self) {
        use Instruction::*;

        todo!("Need to check if MCR[15] has been cleared. If so, stop execution.");

        self.set_pc(self.get_pc() + 1);

        match self.mem.read_word(self.pc).try_into() {
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
                    todo!("Need to check if address is KBSR or KBDR.");
                    self[dr] = self.get_word(self.get_pc().wrapping_add(offset9 as Word));
                    self.set_cc(self[dr]);
                }
                Ldi { dr, offset9 } => {
                    let indir: u16 = self.get_word(self.get_pc().wrapping_add(offset9 as Word));
                    self[dr] = self.get_word(indir);
                    self.set_cc(self[dr]);
                }
                Ldr { dr, base, offset6 } => {
                    self[dr] = self[base].wrapping_add(offset6 as Word);
                    self.set_cc(self[dr]);
                }
                Lea { dr, offset9 } => {
                    self[dr] = self.get_pc().wrapping_add(offset9 as Word);
//                    self.set_cc(self[dr]);        // LEA no longer changes condition codes
                }
                Not { dr, sr } => {
                    self[dr] = !self[sr];
                    self.set_cc(self[dr]);
                }
                Ret => {
                    self.set_pc(self[Reg::R7]);
                }
                Rti => {
                    let psr = self.get_word(PSR);
                    if (psr.bit(15)) == false {
                        self.set_pc(self.get_word(self[Reg::R6]));
                        self[Reg::R6] += 1;
                        let temp = self.get_word(self[Reg::R6]);
                        self[Reg::R6] += 1;
                        self.set_word(PSR, temp);
                    } else {
                        self.handle_exception(0x00);
                    }
                }
                St { sr, offset9 } => {
                    self.set_word(self.get_pc().wrapping_add(offset9 as Word), self[sr]);
                }
                Sti { sr, offset9 } => {
                    let indir: u16 = self.get_word(self.get_pc().wrapping_add(offset9 as Word));
                    self.set_word(indir, self[sr]);
                }
                Str { sr, base, offset6 } => {
                    self.set_word(self[base].wrapping_add(offset6 as Word), self[sr]);
                }
                Trap { trapvec } => {
                    self[Reg::R7] = self.get_pc();
                    self.set_pc(self.get_word(trapvec as u16));
                }
            },
            Err(word) => {
                self.handle_exception(0x01);
            },
        }
    }

    fn set_pc(&mut self, addr: Addr) {
        self.pc = addr;
    }
    fn get_pc(&self) -> Addr {
        self.pc
    }

    fn set_word(&mut self, addr: Addr, word: Word) {
        self.mem.write_word(addr, word)
    }

    fn get_word(&self, addr: Addr) -> Word {
        self.mem.read_word(addr)
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::Instruction;

    // Test that the instructions work
    // Test that the unimplemented instructions do <something>

    fn interp_test_runner<'a, M: Memory, P: Peripherals<'a>>(
        insns: Vec<Instruction>,
        num_steps: Option<usize>,
        regs: [Option<Word>; 8],
        pc: Addr,
        memory_locations: Vec<(Addr, Word)>,
    ) {
        let mut interp = Simulator::<M, P>::default();

        let mut addr = 0x3000;
        for insn in insns {
            interp.set_word(addr, insn.into());
            addr += 2;
        }

        if let Some(num_steps) = num_steps {
            for _ in 0..num_steps {
                interp.step();
            }
        } else {
            // TODO! (run until halted)
        }

        for (idx, r) in regs.iter().enumerate() {
            if let Some(reg_word) = r {
                assert_eq!(interp.get_register(idx.into()), *reg_word);
            }
        }
    }

    #[test]
    //NO-OP do nothing run a cycle test
    fn nop() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::Br { n: true, z: true, p: true, offset11: -1 }],
            Same(1),
            [None, None, None, None, None, None, None, None],
            0x3000,
            vec![]
        )
    }
    //0+1=1 Basic Add
    #[test]
    fn add_reg_test() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::
                AddImm { dr: R1, sr1: R1, imm5: 1 },
                AddReg { dr: 2, sr1: 1, sr2: 0 }],
            Some(1),
            [Some(0), Some(1), Some(1), None, None, None, None, None],
            0x3001,
            vec![]
        )
    }
    //AddImm Test with R0(0) + !
    #[test]
    fn AddImmTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R0, sr1: R0, imm5: 1 }],
            Some(1),
            [1, None, None, None, None, None, None, None],
            0x3001,
            vec![]
        )
    }
    //AndReg Test with R0(1) and R1(2) to R0(expected 3)
    #[test]
    fn AndRegTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R0, sr1: R0, imm5: 1 },
            AddImm { dr: R1, sr1: R1, imm5: 2 },
            AndReg { dr: R0, sr1: R0, sr2: R1 },
            ],
            Some(3),
            [3, 2, None, None, None, None, None, None],
            0x3003,
            vec![]
        )
    }
    //AndImm Test with R1 (1) and 0
    #[test]
    fn AndImmTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R1, sr1: R1, imm5: 1 },
            AndImm { dr: R1, sr1: R1, imm5: 0 },
            ],
            Some(2),
            [0, None, None, None, None, None, None, None],
            0x3002,
            vec![]
        )
    }
    //ST Test which stores 1 into x3001
    fn StTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R0, sr1: R0, imm5: 1 },
            St {sr: R0, offset9: 0 },
            ],
            Some(2),
            [1, None, None, None, None, None, None, None],
            0x3002,
            vec![(0x3001, 1)]
        )
    }
    //LD Test with R0 and memory
    #[test]
    fn LdTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R0, sr1: R0, imm5: 1 },
                St {sr: R0, offset9: 1 },
                Ld { dr: R0, offset9: 0 }
            ],
            Some(3),
            [3001, None, None, None, None, None, None, None],
            0x3003,
            vec![(0x3001, 1)]
        )
    }
    //LDR Test with R0 and memory
    #[test]
    fn LdrTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R0, sr1: R0, imm5: 1 },
                St {sr: R0, offset9: 0 },
                Ldr { dr: R1, offset9: -1 },
            ],
            Some(3),
            [1, 3001, None, None, None, None, None, None],
            0x3003,
            vec![(0x3001, 1)]
        )
    }
    //Load x3000 into R1
    #[test]
    fn LeaTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::Lea { dr: R0, offset9: 0 }],
            Some(1),
            [3000, None, None, None, None, None, None, None],
            0x3001,
            vec![]
        )
    }
    // STR test with offset store into lea using 3000
    #[test]
    fn StrTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::Lea { dr: R1, offset9: 0 },
                Lea { dr: R2, offset9: 1 },
                Str { sr: R2, base: R1, offset6: 1 },
            ],
            Some(3),
            [None, None, None, None, None, None, None, None],
            0x3003,
            vec![(x3004, 3000)]
        )
    }
    //not test
    #[test]
    fn NotTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::AddImm { dr: R0, sr1: R0, imm5: 1 },
                Not { dr: R1, sr: R0 },
            ],
            Some(2),
            [1, 0, None, None, None, None, None, None],
            0x3002,
            vec![]
        )
    }
    //ldi Test using location 3000 and loading value of memory into register, using 3002 and 3001 holding 3000 as reference
    #[test]
    fn LdiTest() {
        interp_test_runner::<MemoryShim, _>(
            vec![Instruction::Lea { dr: R0, offset9: 0 },
                St {sr: R0, offset9: 0 },
                St {sr: R0, offset9: -2 },
                Ldi { dr: R2, offset9: -1 },
            ],
            Some(4),
            [1, None, 3000, None, None, None, None, None],
            0x3004,
            vec![(x3001, 3000), (x3000,3000)]
        )
    }
}
