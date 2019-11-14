// use core::ops::Try;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use lc3_isa::{Addr, Reg::{self, *}, Word};
use lc3_traits::{memory::Memory, peripherals::Peripherals};

use super::mem_mapped::{MemMapped, MemMappedSpecial, PSR, BSP};

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

    fn get_device_reg<M: MemMapped>(&self) -> Result<M, Acv> {
        M::from(self)
    }

    fn set_device_reg<M: MemMapped>(&mut self, value: Word) -> WriteAttempt {
        M::set(self, value)
    }

    fn update_device_reg<M: MemMapped>(&mut self, func: impl FnOnce(M) -> Word) -> WriteAttempt {
        M::update(self, func)
    }

    fn get_special_reg<M: MemMappedSpecial>(&self) -> M { M::from_special(self) }
    fn set_special_reg<M: MemMappedSpecial>(&mut self, value: Word) { M::set_special(self, value) }
    fn update_special_reg<M: MemMappedSpecial>(&mut self, func: impl FnOnce(M) -> Word) { M::update(self, func).unwrap() }
}

// TODO: Swap for Result<Word, Acv>

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Acv;

pub type ReadAttempt = Result<Word, Acv>;

/*#[derive(Copy, Clone, Debug, PartialEq)]
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
}*/


// TODO: Swap for Result<(), Acv>

pub type WriteAttempt = Result<(), Acv>;

// #[derive(Copy, Clone, Debug, PartialEq)]
// pub enum WriteAttempt {
//     Success,
//     Acv,
// }

// impl Try for WriteAttempt {
//     type Ok = ();
//     type Error = ();

//     fn into_result(self) -> Result<Self::Ok, Self::Error> {
//         use WriteAttempt::*;
//         match self {
//             Success => Ok(()),
//             Acv => Err(()),
//         }
//     }

//     fn from_error(_: Self::Error) -> Self { Self::Acv }

//     fn from_ok(_: Self::Ok) -> Self { Self::Success }
// }

// impl WriteAttempt {
//     pub fn ok(self) {
//         self.into_result().unwrap()
//     }
// }

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

impl<'a, M: Memory, P: Peripherals<'a>> Interpreter<'a, M, P> {
    fn set_cc(&mut self, word: Word) {
        <PSR as MemMapped>::from(self).unwrap().set_cc(self, word)
    }

    fn get_cc(&self) -> (bool, bool, bool) {
        self.get_special_reg::<PSR>().get_cc()
    }

    fn push(&mut self, word: Word) -> WriteAttempt {
        self[R6] -= 1;
        self.set_word(self[R6], word)
    }

    // Take notice! This will not modify R6 if the read fails!
    // TODO: Is this correct?
    fn pop(&mut self) -> ReadAttempt {
        let word = self.get_word(self[R6])?;
        self[R6] += 1;

        Ok(word)
    }

    // TODO: Swap result out
    fn push_state(&mut self) -> WriteAttempt {
        // Push the PSR and then the PC so that the PC gets popped first.
        // (Popping the PSR first could trigger an ACV)

        self.push(*self.get_special_reg::<PSR>())
            .and_then(|()| self.push(self.get_pc()))
    }

    fn restore_state(&mut self) -> Result<(), Acv> {
        // Restore the PC and then the PSR.

        self.pop().map(|w| self.set_pc(w))
            .and_then(|()| self.pop().map(|w| self.set_special_reg::<PSR>(w)))
    }

    // Infallible since BSP is 'special'.
    fn swap_stacks(&mut self) {
        let (sp, bsp) = (self[R6], *self.get_special_reg::<BSP>());

        BSP::set_special(self, sp);
        self[R6] = bsp;
    }

    // TODO: execution event = exception, interrupt, or a trap
    // find a better name
    fn prep_for_execution_event(&mut self) {
        let mut psr = self.get_special_reg::<PSR>();

        // If we're in user mode..
        if psr.in_user_mode() {
            // ..switch to supervisor mode..
            psr.to_privileged_mode(self);

            // ..and switch the stacks:
            self.swap_stacks();
        } else {
            // We're assuming if PSR[15] is in supervisor mode, R6 is already
            // the supervisor stack pointer and BSR has the user stack pointer.
        }

        // We're in privileged mode now so this should never panic.
        self.push_state().unwrap();
    }

    // TODO: find a word that generalizes exception and trap...
    // since that's what this handles
    fn handle_exception(&mut self, ex_vec: u8) {
        self.prep_for_execution_event();

        // Go to the exception vector:
        // (this should also not panic)
        self.pc = self.get_word(0x0100 | (Into::<Word>::into(ex_vec))).unwrap();
    }

    fn handle_interrupt(&mut self, int_vec: u8, priority: u8) -> bool {
        // TODO: check that the ordering here is right

        // Make sure that the priority is high enough to interrupt:
        if self.get_special_reg::<PSR>().get_priority() >= priority {
            // Gotta wait.
            return false;
        }

        self.prep_for_execution_event();

        // Go to the interrupt vector:
        // (this should also not panic)
        self.pc = self.get_word(0x0100 | (Into::<Word>::into(int_vec))).unwrap();
        self.get_special_reg::<PSR>().set_priority(self, priority);

        true
    }

    fn is_acv(&self, addr: Word) -> bool {
        // TODO: is `PSR::from_special(self).in_user_mode()` clearer?

        if self.get_special_reg::<PSR>().in_user_mode() {
            (addr < 0x3000) | (addr > 0xFE00)
        } else {
            false
        }
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> InstructionInterpreter for Interpreter<'a, M, P> {
    fn step(&mut self) -> MachineState {
        unimplemented!()
    }

    fn set_pc(&mut self, addr: Addr) {
        self.pc = addr;
    }
    fn get_pc(&self) -> Addr {
        self.pc
    }

    // Checked access:
    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt {
        if self.is_acv(addr) {
            Err(Acv)
        } else {
            Ok(self.set_word_unchecked(addr, word))
        }
    }

    fn get_word(&self, addr: Addr) -> ReadAttempt {
        if self.is_acv(addr) {
            Err(Acv)
        } else {
            Ok(self.get_word_unchecked(addr))
        }
    }

    fn set_word_unchecked(&mut self, addr: Addr, word: Word) {
        if addr >= 0xFE00 {
            // TODO: mem mapped peripherals!
            unimplemented!();
        } else {
            self.memory.write_word(addr, word)
        }
    }

    fn get_word_unchecked(&self, addr: Addr) -> Word {
        if addr >= 0xFE00 {
            // TODO: mem mapped peripherals!
            unimplemented!();
        } else {
            self.memory.read_word(addr)
        }
    }

    fn get_machine_state(&self) -> MachineState {
        self.state
    }

    fn reset(&mut self) {
        unimplemented!();
    }
}


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
