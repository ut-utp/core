use crate::control::Control;
use crate::isa::RegNum;
use crate::memory::Memory;
use crate::peripherals::PeripheralSet;
use crate::peripherals::Peripherals;
use crate::{Addr, Word};
use core::marker::PhantomData;

struct Interpreter<'a, M: Memory, P: Peripherals<'a>> {
    mem: M,
    peripherals: P,
    regs: [Word; 8],
    pc: Word,
    _p: PhantomData<&'a ()>,
}

impl<'a, M: Memory, P: Peripherals<'a>> Default for Interpreter<'a, M, P> {
    fn default() -> Self {
        unimplemented!()
    }
}

pub(crate) trait Interp {
    fn step(&mut self);

    fn set_pc(&mut self, addr: Addr);
    fn get_pc(&self) -> Addr;

    fn set_word(&mut self, addr: Addr, word: Word);
    fn get_word(&self, addr: Addr) -> Word;

    fn get_register(&self, reg: RegNum) -> Word;
    fn set_register(&mut self, reg: RegNum, word: Word);
}

impl From<RegNum> for usize {
    fn from(reg_num: RegNum) -> usize {
        use RegNum::*;
        match reg_num {
            R0 => 0,
            R1 => 1,
            R2 => 2,
            R3 => 3,
            R4 => 4,
            R5 => 5,
            R6 => 6,
            R7 => 7,
        }
    }
}

// Don't do this! impl `TryFrom` instead
impl From<usize> for RegNum {
    fn from(reg_num: usize) -> RegNum {
        use RegNum::*;
        match reg_num {
            0 => R0,
            1 => R1,
            2 => R2,
            3 => R3,
            4 => R4,
            5 => R5,
            6 => R6,
            7 => R7,
            _ => unimplemented!(),
        }
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> Interp for Interpreter<'a, M, P> {
    fn step(&mut self) {
        use crate::isa::Instruction::*;

        // TODO: probably impl TryFrom instead so we don't just crash??
        match self.mem.read_word(self.pc).into() {
            Jsr { offset11 } => {}
            _ => unimplemented!(),
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

    fn get_register(&self, reg: RegNum) -> Word {
        self.regs[Into::<usize>::into(reg)]
    }

    fn set_register(&mut self, reg: RegNum, word: Word) {
        self.regs[Into::<usize>::into(reg)] = word;
    }

    // fn get_state(&self) -> State {
    //     self.state
    // }
}

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
        let mut interp = Interpreter::<M, P>::default();

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
    fn nop() {
        interp_test_runner<MemoryShim, _>(
            vec![Instruction::Br { n: true, z: true, p: true, offset11: -1 }],
            1,
            [None, None, None, None, None, None, None, None],
            0x3000,
            vec![]
        )
    }
}
