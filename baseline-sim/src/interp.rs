// use core::ops::Try;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::sync::atomic::AtomicBool;

use core::ops::{Deref, DerefMut};

use lc3_isa::{
    Addr, Instruction,
    Reg::{self, *},
    Word,
    Instruction,
    MEM_MAPPED_START_ADDR,
    USER_PROGRAM_START_ADDR,
    TRAP_VECTOR_TABLE_START_ADDR,
    INTERRUPT_VECTOR_TABLE_START_ADDR,
    PRIVILEGE_MODE_VIOLATION_EXCEPTION_VECTOR,
    ILLEGAL_OPCODE_EXCEPTION_VECTOR,
    ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR
};
use lc3_traits::{memory::Memory, peripherals::Peripherals};
use lc3_traits::peripherals::{gpio::GpioPinArr, timers::TimerArr};

use super::mem_mapped::{MemMapped, MemMappedSpecial, BSP, PSR};

// TODO: name?
pub trait InstructionInterpreterPeripheralAccess:
    InstructionInterpreter + Deref + DerefMut
where
    for<'a> <Self as Deref>::Target: Peripherals<'a>,
{
    fn get_peripherals(&self) -> &<Self as std::ops::Deref>::Target {
        self.deref()
    }

    fn get_peripherals_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
        self.deref_mut()
    }

    fn get_device_reg<M: MemMapped>(&self) -> Result<M, Acv> {
        M::from(self)
    }

    fn set_device_reg<M: MemMapped>(&mut self, value: Word) -> WriteAttempt {
        M::set(self, value)
    }

    fn update_device_reg<M: MemMapped>(&mut self, func: impl FnOnce(M) -> Word) -> WriteAttempt {
        M::update(self, func)
    }

    fn get_special_reg<M: MemMappedSpecial>(&self) -> M {
        M::from_special(self)
    }
    fn set_special_reg<M: MemMappedSpecial>(&mut self, value: Word) {
        M::set_special(self, value)
    }
    fn update_special_reg<M: MemMappedSpecial>(&mut self, func: impl FnOnce(M) -> Word) {
        M::update(self, func).unwrap()
    }
}

pub trait InstructionInterpreter:
    Index<Reg, Output = Word> + IndexMut<Reg, Output = Word> + Sized
{
    fn step(&mut self) -> MachineState;

    fn set_pc(&mut self, addr: Addr);
    fn get_pc(&self) -> Addr;

    // Checked access:
    fn set_word(&mut self, addr: Addr, word: Word) -> WriteAttempt;
    fn get_word(&self, addr: Addr) -> ReadAttempt;

    fn set_word_unchecked(&mut self, addr: Addr, word: Word);
    fn get_word_unchecked(&self, addr: Addr) -> Word;

    fn get_register(&self, reg: Reg) -> Word {
        self[reg]
    }
    fn set_register(&mut self, reg: Reg, word: Word) {
        self[reg] = word;
    }

    fn get_machine_state(&self) -> MachineState;
    fn reset(&mut self);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Acv;

pub type ReadAttempt = Result<Word, Acv>;

pub type WriteAttempt = Result<(), Acv>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MachineState {
    Running,
    Halted,
}

#[derive(Debug)]
struct PeripheralInterruptFlags {
    gpio: GpioPinArr<AtomicBool>, // No payload; just tell us if a rising edge has happened
    // adc: AdcPinArr<bool>, // We're not going to have Adc Interrupts
    // pwm: PwmPinArr<bool>, // No Pwm Interrupts
    timers: TimerArr<AtomicBool>, // No payload; timers don't actually expose counts anyways
    // clock: bool, // No Clock Interrupt
    input: AtomicBool, // No payload; check KBDR for the current character
    output: AtomicBool, // Technically this has an interrupt, but I have no idea why; UPDATE: it interrupts when it's ready to accept more data
    // display: bool, // Unless we're exposing vsync/hsync or something, this doesn't need an interrupt
}

#[derive(Debug)]
pub struct Interpreter<'a, M: Memory, P: Peripherals<'a>> {
    memory: M,
    peripherals: P,
    flags: PeripheralInterruptFlags,
    regs: [Word; Reg::NUM_REGS],
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

impl<'a, M: Memory, P: Peripherals<'a>> Deref for Interpreter<'a, M, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.peripherals
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> DerefMut for Interpreter<'a, M, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.peripherals
    }
}

impl<'a, M: Memory, P> InstructionInterpreterPeripheralAccess for Interpreter<'a, M, P> where
    for<'b> P: Peripherals<'b>
{
}

impl<'a, M: Memory, P> Interpreter<'a, M, P>
where
    for<'b> P: Peripherals<'b>,
{
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

        self.pop()
            .map(|w| self.set_pc(w))
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

    fn handle_trap(&mut self, trap_vec: u8) {
        self.prep_for_execution_event();

        // Go to the trap vector:
        // (this should also not panic)
        self.pc = self
            .get_word(TRAP_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(trap_vec)))
            .unwrap();
    }

    // TODO: find a word that generalizes exception and trap...
    // since that's what this handles
    fn handle_exception(&mut self, ex_vec: u8) {
        self.prep_for_execution_event();

        // Go to the exception vector:
        // (this should also not panic)
        self.pc = self
            .get_word(INTERRUPT_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(ex_vec)))
            .unwrap();
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
        self.pc = self
            .get_word(INTERRUPT_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(int_vec)))
            .unwrap();
        self.get_special_reg::<PSR>().set_priority(self, priority);

        true
    }

    fn is_acv(&self, addr: Word) -> bool {
        // TODO: is `PSR::from_special(self).in_user_mode()` clearer?

        if self.get_special_reg::<PSR>().in_user_mode() {
            (addr < USER_PROGRAM_START_ADDR) | (addr >= MEM_MAPPED_START_ADDR)
        } else {
            false
        }
    }

    fn instruction_step_inner(&mut self, insn: Instruction) -> Result<(), Acv> {
        use Instruction::*;

        macro_rules! i {
            (PC <- $expr:expr) => {
                self.set_pc($expr);
            };
            (mem[$addr:expr] <- $expr:expr) => {
                self.set_word($addr, $expr)?;
            };
            ($dr:ident <- $expr:expr) => {
                self[$dr] = $expr;
                if insn.sets_condition_codes() { self.set_cc(self[$dr]); }
            };
        }

        macro_rules! I {
            (PC <- $($rest:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut pc: Addr;

                _insn_inner!(pc | $($rest)*);
                i!(PC <- pc);
            }};

            (mem[$($addr:tt)*] <- $($word:tt)*) => {{
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut addr: Addr;
                #[allow(unused_mut)]
                let mut word: Word;

                _insn_inner!(addr | $($addr)*);
                _insn_inner!(word | $($word)*);

                self.set_word(addr, word)?
            }};

            ($dr:ident <- $($rest:tt)*) => {{
                // trace_macros!(true);
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut word: Word;

                // _insn_inner!(word | R[sr1]);
                _insn_inner!(word | $($rest)*);

                i!($dr <- word);
            }};
        }

        macro_rules! _insn_inner_gen {
            ($d:tt) => {
                macro_rules! _insn_inner {
                    ($d nom:ident | R[$d reg:expr] $d ($d rest:tt)*) => { $d nom = self[$d reg]; _insn_inner!($d nom | $d ($d rest)*) };
                    ($d nom:ident | PC $d ($d rest:tt)*) => { $d nom = self.get_pc(); _insn_inner!($d nom | $d ($d rest)*) };
                    ($d nom:ident | mem[$d ($d addr:tt)*] $d ($d rest:tt)*) => {
                        $d nom = self.get_word({
                            let mut _addr_mem: Addr;
                            _insn_inner!(_addr_mem | $d ($d addr)*);
                            _addr_mem
                        }/* as Word*/)?;

                        _insn_inner!($d nom | $d ($d rest)*)
                    };
                    ($d nom:ident | + $d ($d rest:tt)*) => {
                        $d nom = $d nom.wrapping_add(
                        {
                            let mut _rhs_add;
                            _insn_inner!(_rhs_add | $d ($d rest)*);
                            _rhs_add
                        } as Word);
                    };
                    ($d nom:ident | & $d ($d rest:tt)*) => {
                        $d nom = $d nom & {
                            let mut _rhs_and;
                            _insn_inner!(_rhs_and | $d ($d rest)*);
                            _rhs_and
                        } as Word;
                    };
                    ($d nom:ident | ! $d ($d rest:tt)*) => {
                        $d nom = ! {
                            let mut _rhs_not: Word;
                            _insn_inner!(_rhs_not | $d ($d rest)*);
                            _rhs_not
                        } /*as Word*/;
                    };
                    ($d nom:ident | $d ident:ident $d ($d rest:tt)*) => {
                        $d nom = $d ident; _insn_inner!($d nom | $d ($d rest)*)
                    };
                    ($d nom:ident | ) => {};
                }
            }
        }

        match insn {
            AddReg { dr, sr1, sr2 } => {
                i!(dr <- self[sr1].wrapping_add(self[sr2]));
                // I!(dr <- R[sr1] + R[sr2])
            }
            AddImm { dr, sr1, imm5 } => {
                i!(dr <- self[sr1].wrapping_add(imm5 as Word));
                // I!(dr <- R[sr1] + imm5)
            }
            AndReg { dr, sr1, sr2 } => {
                i!(dr <- self[sr1] & self[sr2]);
                // I!(dr <- R[sr1] & R[sr2])
            }
            AndImm { dr, sr1, imm5 } => {
                i!(dr <- self[sr1] & (imm5 as Word));
                // I!(dr <- R[sr1] & imm5)
            }
            Br { n, z, p, offset9 } => {
                let (cc_n, cc_z, cc_p) = self.get_cc();
                if n & cc_n || z & cc_z || p & cc_p {
                    i!(PC <- self.get_pc().wrapping_add(offset9 as Word));

                    // I!(PC <- PC + offset9)
                }
            }
            Jmp { base: R7 } | Ret => {
                i!(PC <- self[R7]);
                // I!(PC <- R[R7])
            }
            Jmp { base } => {
                i!(PC <- self[base]);
                // I!(PC <- R[base])
            }
            Jsr { offset11 } => {
                self[R7] = self.get_pc();
                i!(PC <- self.get_pc().wrapping_add(offset11 as Word));

                // I!(R7 <- PC);
                // I!(PC <- PC + offset11)
            }
            Jsrr { base } => {
                // TODO: add a test where base _is_ R7!!
                let (pc, new_pc) = (self.get_pc(), self[base]);
                i!(PC <- new_pc);
                i!(R7 <- pc);

                // I!(PC <- new_pc);
                // I!(R7 <- pc)
            }
            Ld { dr, offset9 } => {
                let addr = self.get_pc().wrapping_add(offset9 as Word);
                i!(dr <- self.get_word(addr)?);

                // I!(dr <- mem[PC + offset9])
            }
            Ldi { dr, offset9 } => {
                // mem[mem[pc + offset]]
                let addr = self.get_pc().wrapping_add(offset9 as Word);
                let addr = self.get_word(addr)?;

                i!(dr <- self.get_word(addr)?);

                // I!(dr <- mem[mem[PC + offset9]])
            }
            Ldr { dr, base, offset6 } => {
                // mem[base + offset6]
                let addr = self[base].wrapping_add(offset6 as Word);
                i!(dr <- self.get_word(addr)?);

                // I!(dr <- mem[R[base] + offset6])
            }
            Lea { dr, offset9 } => {
                i!(dr <- self.get_pc().wrapping_add(offset9 as Word));

                // I!(dr <- PC + offset9)
            }
            Not { dr, sr } => {
                i!(dr <- !self[sr]);

                // I!(dr <- !R[sr])
            }
            Rti => {
                unimplemented!();
            }
            St { sr, offset9 } => {
                i!(mem[self.get_pc().wrapping_add(offset9 as Word)] <- self[sr]);

                I!(mem[PC + offset9] <- R[sr])
            }
            Sti { sr, offset9 } => {
                i!(mem[self.get_word(self.get_pc().wrapping_add(offset9 as Word))?] <- self[sr]);

                // I!(mem[mem[PC + offset9]] <- R[sr])
            }
            Str { sr, base, offset6 } => {
                i!(mem[self[base].wrapping_add(offset6 as Word)] <- self[sr]);

                // I!(mem[R[base] + offset6] <- R[sr])
            }
            Trap { trapvec } => {
                unimplemented!();
            }
        }

        Ok(())
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> InstructionInterpreter for Interpreter<'a, M, P> {
    fn step(&mut self) -> MachineState {
        if let state @ MachineState::Halted = self.get_machine_state() {
            return state;
        }

        // Increment PC (state 18):
        let current_pc = self.get_pc();
        self.set_pc(current_pc.wrapping_add(1)); // TODO: ???

        // TODO: Peripheral interrupt stuff

        // match match self.get_word(current_pc) {
        //     Ok(word) => match word.try_into() {
        //         Ok(insn) => self.instruction_step_inner(insn),
        //         Err(_) => { self.handle_exception(ILLEGAL_OPCODE_EXCEPTION_VECTOR); Ok(()) }
        //     }
        //     Err(Acv) => {
        //         // TODO: what do
        //         // panic!();
        //         Err(Acv)
        //     }
        // } {
        //     Ok(_) => {}
        //     Err(Acv) => { self.handle_exception(ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR); }
        // }

        match self.get_word(current_pc).and_then(|w| {
            match w.try_into() {
                Ok(insn) => self.instruction_step_inner(insn),
                Err(_) => { self.handle_exception(ILLEGAL_OPCODE_EXCEPTION_VECTOR); Ok(()) }
            }
        }) {
            Ok(()) => {},
            // Access control violation: triggered when getting the current instruction or when executing it
            Err(Acv) => self.handle_exception(ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR),
        }

        self.get_machine_state()
    }

    fn set_pc(&mut self, addr: Addr) { self.pc = addr; }
    fn get_pc(&self) -> Addr { self.pc }

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

    // Unchecked access:
    fn set_word_unchecked(&mut self, addr: Addr, word: Word) {
        if addr >= MEM_MAPPED_START_ADDR {
            // TODO: mem mapped peripherals!
            unimplemented!();
        } else {
            self.memory.write_word(addr, word)
        }
    }

    fn get_word_unchecked(&self, addr: Addr) -> Word {
        if addr >= MEM_MAPPED_START_ADDR {
            // TODO: mem mapped peripherals!
            unimplemented!();
        } else {
            self.memory.read_word(addr)
        }
    }

    fn get_machine_state(&self) -> MachineState { self.state }

    fn reset(&mut self) {
        // TODO!
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
