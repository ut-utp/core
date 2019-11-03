use core::ops::Try;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use lc3_isa::{Addr, Reg, Word};
use lc3_traits::{memory::Memory, peripherals::Peripherals};

use super::mem_mapped::MemMapped;

pub trait InstructionInterpreter:
    Index<Reg, Output = Word>
    + IndexMut<Reg, Output = Word>
    + Sized
{
    fn step(&mut self) -> MachineState;

    fn set_pc(&mut self, addr: Addr);
    fn get_pc(&self) -> Addr;

    // Checked access:
    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt;
    fn get_word(&self, addr: Addr) -> ReadAttempt;

    fn set_word_unchecked(&mut self, addr: Addr, word: Word);
    fn get_word_unchecked(&self, addr: Addr) -> Word;

    fn get_register(&self, reg: Reg) -> Word { self[reg] }
    fn set_register(&mut self, reg: Reg, word: Word) { self[reg] = word; }

    fn get_machine_state(&self) -> MachineState;
    fn reset(&mut self);

    fn get_device_reg<M: MemMapped>(&self) -> Result<M, ()> {
        M::from(self)
    }

    fn set_device_reg<M: MemMapped>(&mut self, value: Word) -> WriteAttempt {
        M::set(self, value)
    }

    fn update_device_reg<M: MemMapped>(&mut self, func: impl FnOnce(M) -> Word) -> WriteAttempt {
        M::update(self, func)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReadAttempt {
    Success(Word),
    Acv,
}

impl Try for ReadAttempt {
    type Ok = Word;
    type Error = ();

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        use ReadAttempt::*;
        match self {
            Success(w) => Ok(w),
            Acv => Err(()),
        }
    }

    fn from_error(_: Self::Error) -> Self { Self::Acv }

    fn from_ok(word: Self::Ok) -> Self { Self::Success(word) }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WriteAttempt {
    Success,
    Acv,
}

impl Try for WriteAttempt {
    type Ok = ();
    type Error = ();

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        use WriteAttempt::*;
        match self {
            Success => Ok(()),
            Acv => Err(()),
        }
    }

    fn from_error(_: Self::Error) -> Self { Self::Acv }

    fn from_ok(_: Self::Ok) -> Self { Self::Success }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MachineState {
    Running,
    Halted,
}

// struct PeripheralState {
//     input_character_ready: Cell<bool>
// }

pub struct Interpreter<'a, M: Memory, P: Peripherals<'a>> {
    memory: M,
    peripherals: P,
    regs: [Word; 8],
    pc: Word,
    state: MachineState,
    _p: PhantomData<&'a ()>,
}

impl<'a, M: Memory, P: Peripherals<'a>> Index<Reg> for Interpreter<'a, M, P> {
    type Output = Word;

    fn index(&self, reg: Reg) -> &Self::Output {
        &self.regs[TryInto::<usize>::try_into(Into::<u8>::into(reg)).unwrap()]
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> IndexMut<Reg> for Interpreter<'a, M, P> {
    fn index_mut(&mut self, reg: Reg) -> &mut Self::Output {
        &mut self.regs[TryInto::<usize>::try_into(Into::<u8>::into(reg)).unwrap()]
    }
}

// impl<'a, M: Memory, P: Peripherals<'a>> Index<Addr> for Interpreter<'a, M, P> {
//     type Output = ReadAttempt;

//     fn index(&self, addr: Addr) -> &Self::Output {
//         &self.get_word(addr)
//     }
// }

// impl<'a, M: Memory, P: Peripherals<'a>> Interpreter<'a, M, P> {
//     fn set_cc(&mut self, word: Word) {

//     }
// }

// impl<'a, M: Memory, P: Peripherals<'a>> InstructionInterpreter for Interpreter<'a, M, P> {
//     fn step(&mut self) -> MachineState;

//     fn set_pc(&mut self, addr: Addr);
//     fn get_pc(&self) -> Addr;

//     // Checked access:
//     fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt;
//     fn get_word(&self, addr: Addr) -> ReadAttempt;

//     fn set_word_unchecked(&mut self, addr: Addr, word: Word);
//     fn get_word_unchecked(&self, addr: Addr) -> Word;

//     fn get_register(&self, reg: Reg) -> Word { self[reg] }
//     fn set_register(&mut self, reg: Reg, word: Word) { self[reg] = word; }

//     fn get_machine_state(&self) -> MachineState;
//     fn reset(&mut self);
// }


// struct Interpter<'a, M: Memory, P: Periperals<'a>> {
//     memory: M,
//     peripherals: P,
//     regs: [Word; 8],
//     pc: Word,
//     _p: PhantomData<&'a ()>,
// }

// impl<'a, M: Memory, P: Peripherals<'a>> Default for Interpter<'a, M, P> {
//     fn default() -> Self {
//         Self {
//             memory:
//         }
//     }
// }
