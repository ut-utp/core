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
pub const BSP: Addr = 0xFFFA; // TODO: when is this used!!!
pub const PSR: Addr = 0xFFFC;
pub const MCR: Addr = 0xFFFE;

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
}

impl<'a, M: Memory, P: Peripherals<'a>> Sim for Simulator<'a, M, P> {
    fn step(&mut self) {
        use Instruction::*;

        self.set_pc(self.get_pc() + 1);

        match self.mem.read_word(self.pc).try_into() {
            Ok(ins) => match ins {
                AddReg { dr, sr1, sr2 } => {
                    self[dr] = self[sr1].wrapping_add(self[sr2]);
                    self.set_cc(self[dr]);
                }
                AddImm { dr, sr1, imm5 } => {
                    self[dr] = self[sr1] + imm5 as u16;
                    self.set_cc(self[dr]);
                }
                AndReg { dr, sr1, sr2 } => {}
                AndImm { dr, sr1, imm5 } => {}
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
                Ld { dr, offset9 } => {}
                Ldi { dr, offset9 } => {}
                Ldr { dr, base, offset6 } => {}
                Lea { dr, offset9 } => {}
                Not { dr, sr } => {}
                Ret => {
                    self.set_pc(self[Reg::R7]);
                }
                Rti => {}
                St { sr, offset9 } => {}
                Sti { sr, offset9 } => {}
                Str { sr, base, offset6 } => {}
                Trap { trapvec } => {}
            },
            Err(word) => {}//todo!("exception!?"),
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
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::isa::Instruction;
//
//    // Test that the instructions work
//    // Test that the unimplemented instructions do <something>
//
//    fn interp_test_runner<'a, M: Memory, P: Peripherals<'a>>(
//        insns: Vec<Instruction>,
//        num_steps: Option<usize>,
//        regs: [Option<Word>; 8],
//        pc: Addr,
//        memory_locations: Vec<(Addr, Word)>,
//    ) {
//        let mut interp = Simulator::<M, P>::default();
//
//        let mut addr = 0x3000;
//        for insn in insns {
//            interp.set_word(addr, insn.into());
//            addr += 2;
//        }
//
//        if let Some(num_steps) = num_steps {
//            for _ in 0..num_steps {
//                interp.step();
//            }
//        } else {
//            // TODO! (run until halted)
//        }
//
//        for (idx, r) in regs.iter().enumerate() {
//            if let Some(reg_word) = r {
//                assert_eq!(interp.get_register(idx.into()), *reg_word);
//            }
//        }
//    }
//
//    #[test]
//    fn nop() {
//        interp_test_runner<MemoryShim, _>(
//            vec![Instruction::Br { n: true, z: true, p: true, offset11: -1 }],
//            1,
//            [None, None, None, None, None, None, None, None],
//            0x3000,
//            vec![]
//        )
//    }
//}