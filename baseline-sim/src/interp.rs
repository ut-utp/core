// use core::ops::Try;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};
use core::sync::atomic::AtomicBool;

use core::ops::{Deref, DerefMut};

use lc3_isa::{
    Addr, Instruction,
    Reg::{self, *},
    Word, ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR, ILLEGAL_OPCODE_EXCEPTION_VECTOR,
    INTERRUPT_VECTOR_TABLE_START_ADDR, MEM_MAPPED_START_ADDR,
    PRIVILEGE_MODE_VIOLATION_EXCEPTION_VECTOR, TRAP_VECTOR_TABLE_START_ADDR,
    USER_PROGRAM_START_ADDR,
};
use lc3_traits::peripherals::{gpio::GpioPinArr, timers::TimerArr};
use lc3_traits::{memory::Memory, peripherals::Peripherals};

use super::mem_mapped::{MemMapped, MemMappedSpecial /*, BSP, PSR*/};

// TODO: Break up this file!

// TODO: name?
pub trait InstructionInterpreterPeripheralAccess<'a>:
    InstructionInterpreter + Deref + DerefMut
where
    <Self as Deref>::Target: Peripherals<'a>,
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

    fn set_word_force_memory_backed(&mut self, addr: Addr, word: Word);
    fn get_word_force_memory_backed(&self, addr: Addr) -> Word;

    fn get_register(&self, reg: Reg) -> Word {
        self[reg]
    }
    fn set_register(&mut self, reg: Reg, word: Word) {
        self[reg] = word;
    }

    fn get_machine_state(&self) -> MachineState;
    fn reset(&mut self);
    fn halt(&mut self); // TODO: have the MCR set this, etc.
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

impl MachineState {
    const fn new() -> Self {
        MachineState::Halted
    }
}

impl Default for MachineState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct PeripheralInterruptFlags {
    gpio: GpioPinArr<AtomicBool>, // No payload; just tell us if a rising edge has happened
    // adc: AdcPinArr<bool>, // We're not going to have Adc Interrupts
    // pwm: PwmPinArr<bool>, // No Pwm Interrupts
    timers: TimerArr<AtomicBool>, // No payload; timers don't actually expose counts anyways
    // clock: bool, // No Clock Interrupt
    input: AtomicBool, // No payload; check KBDR for the current character
    output: AtomicBool, // Technically this has an interrupt, but I have no idea why; UPDATE: it interrupts when it's ready to accept more data
                        // display: bool, // Unless we're exposing vsync/hsync or something, this doesn't need an interrupt
}

impl PeripheralInterruptFlags {
    const fn new() -> Self {
        macro_rules! b {
            () => {
                AtomicBool::new(false)
            };
        }

        // TODO: make this less gross..
        Self {
            gpio: GpioPinArr([b!(), b!(), b!(), b!(), b!(), b!(), b!(), b!()]),
            timers: TimerArr([b!(), b!()]),
            input: AtomicBool::new(false),
            output: AtomicBool::new(false),
        }
    }
}

impl Default for PeripheralInterruptFlags {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Either find a `core` replacement for this or pull it out into a `util`
// mod or something.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnedOrRef<'a, T> {
    Owned(T),
    Ref(&'a T),
}

impl<T: Default> Default for OwnedOrRef<'_, T> {
    fn default() -> Self {
        Self::Owned(Default::default())
    }
}

impl<T> Deref for OwnedOrRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        use OwnedOrRef::*;

        match self {
            Owned(inner) => inner,
            Ref(inner) => inner,
        }
    }
}

// impl<T> DerefMut for OwnedOrRef<'_, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         use OwnedOrRef::*;

//         match self {
//             Owned(inner) => inner,
//             Ref(inner) => inner,
//         }
//     }
// }

#[derive(Debug, Default)]
pub struct Interpreter<'a, M: Memory, P: Peripherals<'a>> {
    memory: M,
    peripherals: P,
    flags: OwnedOrRef<'a, PeripheralInterruptFlags>,
    regs: [Word; Reg::NUM_REGS],
    pc: Word, //TODO: what should the default for this be
    state: MachineState,
}

// impl<'a, M: Memory, P> Default for Interpreter<'a, M, P>
// where for <'p> P: Peripherals<'p> {
//     fn default() -> Self {
//         Self {
//             memory: Default::default(),
//             peripherals: P
//         }
//     }
// }

#[derive(Debug)]
pub struct Set;
#[derive(Debug)]
pub struct NotSet;

#[derive(Debug)]
struct InterpreterBuilderData<'a, M: Memory, P>
where
    P: Peripherals<'a>,
{
    memory: Option<M>,
    peripherals: Option<P>,
    flags: Option<OwnedOrRef<'a, PeripheralInterruptFlags>>,
    regs: Option<[Word; Reg::NUM_REGS]>,
    pc: Option<Word>,
    state: Option<MachineState>,
}

#[derive(Debug)]
pub struct InterpreterBuilder<
    'a,
    M: Memory,
    P,
    Mem = NotSet,
    Perip = NotSet,
    Flags = NotSet,
    Regs = NotSet,
    Pc = NotSet,
    State = NotSet,
> where
    P: Peripherals<'a>,
{
    data: InterpreterBuilderData<'a, M, P>,
    _mem: PhantomData<&'a Mem>,
    _perip: PhantomData<&'a Perip>,
    _flags: PhantomData<&'a Flags>,
    _regs: PhantomData<&'a Regs>,
    _pc: PhantomData<&'a Pc>,
    _state: PhantomData<&'a State>,
}

impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    fn with_data(data: InterpreterBuilderData<'a, M, P>) -> Self {
        Self {
            data,
            _mem: PhantomData,
            _perip: PhantomData,
            _flags: PhantomData,
            _regs: PhantomData,
            _pc: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<'a, M: Memory, P> InterpreterBuilder<'a, M, P, NotSet, NotSet, NotSet, NotSet, NotSet, NotSet>
where
    P: Peripherals<'a>,
{
    pub fn new() -> Self {
        Self {
            data: InterpreterBuilderData {
                memory: None,
                peripherals: None,
                flags: None,
                regs: None,
                pc: None,
                state: None,
            },
            _mem: PhantomData,
            _perip: PhantomData,
            _flags: PhantomData,
            _regs: PhantomData,
            _pc: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<'a, M: Memory + Default, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_defaults(self) -> InterpreterBuilder<'a, M, P, Set, Set, Set, Set, Set, Set> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            memory: Some(Default::default()),
            peripherals: Some(Default::default()),
            flags: Some(Default::default()),
            regs: Some(Default::default()),
            pc: Some(Default::default()),
            state: Some(Default::default()),
        })
    }
}

// impl<'a, M: Memory, P, Perip, Flags, Regs, Pc, State> InterpreterBuilder<'a, M, P, NotSet, Perip, Flags, Regs, Pc, State>
impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_memory(
        self,
        memory: M,
    ) -> InterpreterBuilder<'a, M, P, Set, Perip, Flags, Regs, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            memory: Some(memory),
            ..self.data
        })
    }
}

// impl<'a, M: Memory + Default, P, Perip, Flags, Regs, Pc, State> InterpreterBuilder<'a, M, P, NotSet, Perip, Flags, Regs, Pc, State>
impl<'a, M: Memory + Default, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_default_memory(
        self,
    ) -> InterpreterBuilder<'a, M, P, Set, Perip, Flags, Regs, Pc, State> {
        self.with_memory(Default::default())
    }
}

// impl<'a, M: Memory, P, Mem, Flags, Regs, Pc, State> InterpreterBuilder<'a, M, P, Mem, NotSet, Flags, Regs, Pc, State>
impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_peripherals(
        self,
        peripherals: P,
    ) -> InterpreterBuilder<'a, M, P, Mem, Set, Flags, Regs, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            peripherals: Some(peripherals),
            ..self.data
        })
    }

    pub fn with_default_peripherals(
        self,
    ) -> InterpreterBuilder<'a, M, P, Mem, Set, Flags, Regs, Pc, State> {
        self.with_peripherals(Default::default())
    }
}

// impl<'a, M: Memory, P, Mem, Perip, Regs, Pc, State> InterpreterBuilder<'a, M, P, Mem, Perip, NotSet, Regs, Pc, State>
impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_interrupt_flags_by_ref(
        self,
        flags: &'a PeripheralInterruptFlags,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Set, Regs, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            flags: Some(OwnedOrRef::Ref(&flags)),
            ..self.data
        })
    }

    pub fn with_owned_interrupt_flags(
        self,
        flags: PeripheralInterruptFlags,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Set, Regs, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            flags: Some(OwnedOrRef::Owned(flags)),
            ..self.data
        })
    }

    pub fn with_default_interrupt_flags(
        self,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Set, Regs, Pc, State> {
        self.with_owned_interrupt_flags(Default::default())
    }
}

// TODO: do we want to allow people to set the starting register values?
// impl<'a, M: Memory, P, Mem, Perip, Flags, Pc, State> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, NotSet, Pc, State>
impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_regs(
        self,
        regs: [Word; Reg::NUM_REGS],
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Set, Pc, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            regs: Some(regs),
            ..self.data
        })
    }

    pub fn with_default_regs(
        self,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Set, Pc, State> {
        self.with_regs(Default::default())
    }
}

// TODO: do we want to allow people to set the starting pc?
// impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, State> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, NotSet, State>
impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_pc(
        self,
        pc: Word,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Set, State> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            pc: Some(pc),
            ..self.data
        })
    }

    pub fn with_default_pc(
        self,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Set, State> {
        self.with_pc(Default::default())
    }
}

// TODO: do we want to allow people to set the starting machine state?
// impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, NotSet>
impl<'a, M: Memory, P, Mem, Perip, Flags, Regs, Pc, State>
    InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, State>
where
    P: Peripherals<'a>,
{
    pub fn with_state(
        self,
        state: MachineState,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, Set> {
        InterpreterBuilder::with_data(InterpreterBuilderData {
            state: Some(state),
            ..self.data
        })
    }

    pub fn with_default_state(
        self,
    ) -> InterpreterBuilder<'a, M, P, Mem, Perip, Flags, Regs, Pc, Set> {
        self.with_state(Default::default())
    }
}

impl<'a, M: Memory, P> InterpreterBuilder<'a, M, P, Set, Set, Set, Set, Set, Set>
where
    P: Peripherals<'a>,
{
    pub fn build(self) -> Interpreter<'a, M, P> {
        Interpreter::new(
            self.data.memory.unwrap(),
            self.data.peripherals.unwrap(),
            self.data.flags.unwrap(),
            self.data.regs.unwrap(),
            self.data.pc.unwrap(),
            self.data.state.unwrap(),
        )
    }
}

impl<'a, M: Memory, P: Peripherals<'a>> Interpreter<'a, M, P> {
    pub fn new(
        memory: M,
        peripherals: P,
        flags: OwnedOrRef<'a, PeripheralInterruptFlags>,
        regs: [Word; Reg::NUM_REGS],
        pc: Word,
        state: MachineState,
    ) -> Self {
        // TODO: propagate flags to the peripherals!

        let interp = Self {
            memory,
            peripherals,
            flags,
            regs,
            pc,
            state,
        };

        // interp.reset(); // TODO: should we? won't that negate setting the regs and pc and stuff?

        interp
    }
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

impl<'a, M: Memory, P: Peripherals<'a>> InstructionInterpreterPeripheralAccess<'a> for Interpreter<'a, M, P>
{
}

impl<'a, M: Memory, P: Peripherals<'a>> Interpreter<'a, M, P>
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

        // Go to the trap routine:
        // (this should also not panic)
        self.pc = self
            .get_word(TRAP_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(trap_vec)))
            .unwrap();
    }

    // TODO: find a word that generalizes exception and trap...
    // since that's what this handles
    fn handle_exception(&mut self, ex_vec: u8) {
        self.prep_for_execution_event();

        // Go to the exception routine:
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

        self.get_special_reg::<PSR>().set_priority(self, priority);
        self.handle_exception(int_vec);

        true
        // self.prep_for_execution_event();

        // // Go to the interrupt routine:
        // // (this should also not panic)
        // self.pc = self
        //     .get_word(INTERRUPT_VECTOR_TABLE_START_ADDR | (Into::<Word>::into(int_vec)))
        //     .unwrap();

        // true
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
                if insn.sets_condition_codes() {
                    self.set_cc(self[$dr]);
                }
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
                _insn_inner_gen!($);
                #[allow(unused_mut)]
                let mut word: Word;

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
            AddReg { dr, sr1, sr2 } => I!(dr <- R[sr1] + R[sr2]),
            AddImm { dr, sr1, imm5 } => I!(dr <- R[sr1] + imm5),
            AndReg { dr, sr1, sr2 } => I!(dr <- R[sr1] & R[sr2]),
            AndImm { dr, sr1, imm5 } => I!(dr <- R[sr1] & imm5),
            Br { n, z, p, offset9 } => {
                let (cc_n, cc_z, cc_p) = self.get_cc();
                if n & cc_n || z & cc_z || p & cc_p {
                    I!(PC <- PC + offset9)
                }
            }
            Jmp { base: R7 } | Ret => I!(PC <- R[R7]),
            Jmp { base } => I!(PC <- R[base]),
            Jsr { offset11 } => {
                I!(R7 <- PC);
                I!(PC <- PC + offset11)
            }
            Jsrr { base } => {
                // TODO: add a test where base _is_ R7!!
                let (pc, new_pc) = (self.get_pc(), self[base]);
                I!(PC <- new_pc);
                I!(R7 <- pc)
            }
            Ld { dr, offset9 } => I!(dr <- mem[PC + offset9]),
            Ldi { dr, offset9 } => I!(dr <- mem[mem[PC + offset9]]),
            Ldr { dr, base, offset6 } => I!(dr <- mem[R[base] + offset6]),
            Lea { dr, offset9 } => I!(dr <- PC + offset9),
            Not { dr, sr } => I!(dr <- !R[sr]),
            Rti => {
                if self.get_special_reg::<PSR>().in_privileged_mode() {
                    // This should never panic:
                    self.restore_state().unwrap();

                    // If we've gone back to user mode..
                    if self.get_special_reg::<PSR>().in_user_mode() {
                        // ..swap the stack pointers.
                        self.swap_stacks();
                    }
                } else {
                    // If RTI is called from user mode, raise the privilege mode
                    // exception:
                    self.handle_exception(PRIVILEGE_MODE_VIOLATION_EXCEPTION_VECTOR)
                }
            }
            St { sr, offset9 } => I!(mem[PC + offset9] <- R[sr]),
            Sti { sr, offset9 } => I!(mem[mem[PC + offset9]] <- R[sr]),
            Str { sr, base, offset6 } => I!(mem[R[base] + offset6] <- R[sr]),
            Trap { trapvec } => self.handle_trap(trapvec),
        }

        Ok(())
    }
}

use super::mem_mapped::{BSP, DDR, DSR, KBDR, KBSR, PSR};
use super::mem_mapped::{
    G0CR, G0DR, G1CR, G1DR, G2CR, G2DR, G3CR, G3DR, G4CR, G4DR, G5CR, G5DR, G6CR, G6DR, G7CR, G7DR,
};

impl<'a, M: Memory, P: Peripherals<'a>> InstructionInterpreter for Interpreter<'a, M, P>
{
    fn step(&mut self) -> MachineState {
        if let state @ MachineState::Halted = self.get_machine_state() {
            return state;
        }

        // Increment PC (state 18):
        let current_pc = self.get_pc();
        self.set_pc(current_pc.wrapping_add(1)); // TODO: ???

        // TODO: Peripheral interrupt stuff

        match self.get_word(current_pc).and_then(|w| match w.try_into() {
            Ok(insn) => self.instruction_step_inner(insn),
            Err(_) => {
                self.handle_exception(ILLEGAL_OPCODE_EXCEPTION_VECTOR);
                Ok(())
            }
        }) {
            Ok(()) => {}
            // Access control violation: triggered when getting the current instruction or when executing it
            Err(Acv) => self.handle_exception(ACCESS_CONTROL_VIOLATION_EXCEPTION_VECTOR),
        }

        self.get_machine_state()
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

    // Unchecked access:
    #[forbid(unreachable_patterns)]
    fn set_word_unchecked(&mut self, addr: Addr, word: Word) {
        if addr >= MEM_MAPPED_START_ADDR {
            macro_rules! devices {
                ($($dev:ty),*) => {
                    match addr {
                        $(<$dev as MemMapped>::ADDR => self.set_device_reg::<$dev>(word).unwrap(),)*
                        _ => unimplemented!() // TODO: make a sane handler?
                    }
                };
            }

            devices!(
                KBSR, KBDR, DSR, DDR, BSP, PSR, G0CR, G0DR, G1CR, G1DR, G2CR, G2DR, G3CR, G3DR,
                G4CR, G4DR, G5CR, G5DR, G6CR, G6DR, G7CR, G7DR
            )
        } else {
            self.set_word_force_memory_backed(addr, word)
        }
    }

    #[forbid(unreachable_patterns)]
    fn get_word_unchecked(&self, addr: Addr) -> Word {
        if addr >= MEM_MAPPED_START_ADDR {
            // TODO: mem mapped peripherals!
            macro_rules! devices {
                ($($dev:ty),*) => {
                    match addr {
                        $(<$dev as MemMapped>::ADDR => *self.get_device_reg::<$dev>().unwrap(),)*
                        _ => unimplemented!() // TODO: make a sane handler?
                    }
                };
            }

            devices!(
                KBSR, KBDR, DSR, DDR, BSP, PSR, G0CR, G0DR, G1CR, G1DR, G2CR, G2DR, G3CR, G3DR,
                G4CR, G4DR, G5CR, G5DR, G6CR, G6DR, G7CR, G7DR
            )
        } else {
            self.get_word_force_memory_backed(addr)
        }
    }

    fn set_word_force_memory_backed(&mut self, addr: Addr, word: Word) {
        self.memory.write_word(addr, word)
    }

    fn get_word_force_memory_backed(&self, addr: Addr) -> Word {
        self.memory.read_word(addr)
    }


    fn get_machine_state(&self) -> MachineState {
        self.state
    }

    fn reset(&mut self) {
        self.pc = 0; // TODO?
        self.set_cc(0);

        // TODO!
        // unimplemented!();
        self.state = MachineState::Running;
    }

    fn halt(&mut self) {
        self.state = MachineState::Halted;
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
