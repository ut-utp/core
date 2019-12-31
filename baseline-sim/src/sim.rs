use crate::interp::{InstructionInterpreter, InstructionInterpreterPeripheralAccess, MachineState};

use lc3_isa::{Addr, Reg, Word};
use lc3_traits::control::{Control, Event, State};
use lc3_traits::control::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use lc3_traits::control::rpc::{EventFutureSharedStatePorcelain, SimpleEventFutureSharedState, EventFuture};
use lc3_traits::error::Error;
use lc3_traits::memory::MemoryMiscError;
use lc3_traits::peripherals::adc::{Adc, AdcPinArr, AdcReadError, AdcState};
use lc3_traits::peripherals::clock::Clock;
use lc3_traits::peripherals::gpio::{Gpio, GpioPinArr, GpioReadError, GpioState};
use lc3_traits::peripherals::pwm::{Pwm, PwmPinArr, PwmState};
use lc3_traits::peripherals::timers::{TimerArr, TimerState, Timers};
use lc3_traits::peripherals::Peripherals;

use crate::mem_mapped::{MemMapped, KBDR};

// use core::future::Future;
use core::marker::PhantomData;
use core::ops::Deref;
// use core::pin::Pin;
// use core::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct Simulator<'a, 's, I: InstructionInterpreter + InstructionInterpreterPeripheralAccess<'a>, S: EventFutureSharedStatePorcelain = SimpleEventFutureSharedState>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    interp: I,
    breakpoints: [Option<Addr>; MAX_BREAKPOINTS],
    watchpoints: [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS], // TODO: change to throw these when the location being watched to written to; not just when the value is changed...
    num_set_breakpoints: usize,
    num_set_watchpoints: usize,
    state: State,
    shared_state: Option<&'s S>,
    _i: PhantomData<&'a ()>,
}

impl<'a, 's, I: InstructionInterpreterPeripheralAccess<'a> + Default, S: EventFutureSharedStatePorcelain> Default for Simulator<'a, 's, I, S>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<'a, 's, I: InstructionInterpreterPeripheralAccess<'a>, S: EventFutureSharedStatePorcelain> Simulator<'a, 's, I, S>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    // No longer public.
    fn new(interp: I) -> Self {
        Self {
            interp,
            breakpoints: [None; MAX_BREAKPOINTS],
            watchpoints: [None; MAX_MEMORY_WATCHPOINTS],
            num_set_breakpoints: 0,
            num_set_watchpoints: 0,
            state: State::Paused,
            shared_state: None,
            _i: PhantomData,
        }
    }

    pub fn new_with_state(interp: I, state: &'s S) -> Self {
        let mut sim = Self::new(interp);
        sim.set_shared_state(state);

        sim
    }

    pub fn set_shared_state(&mut self, state: &'s S) {
        self.shared_state = Some(state);
    }
}

// impl<'a, I: InstructionInterpreterPeripheralAccess<'a>> Simulator<'a, I>
// where
//     <I as Deref>::Target: Peripherals<'a>,
// {
//     pub fn get_interpreter(&mut self) -> &mut I {
//         &mut self.interp
//     }
// }

impl<'a, 's, I: InstructionInterpreterPeripheralAccess<'a>, S: EventFutureSharedStatePorcelain> Control for Simulator<'a, 's, I, S>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    type EventFuture = EventFuture<'s, S>;

    fn get_pc(&self) -> Addr {
        self.interp.get_pc()
    }

    fn set_pc(&mut self, addr: Addr) {
        self.interp.set_pc(addr)
    }

    fn get_register(&self, reg: Reg) -> Word {
        self.interp.get_register(reg)
    }

    fn set_register(&mut self, reg: Reg, data: Word) {
        self.interp.set_register(reg, data)
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.interp.set_word_unchecked(addr, word)
    }

    fn read_word(&self, addr: Addr) -> Word {
        // Our one stateful read
        // TODO: banish this from the codebase
        if addr == <KBDR as MemMapped>::ADDR {
            return 0;
        }

        self.interp.get_word_unchecked(addr)
    }

    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        self.interp.commit_memory()
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        if self.num_set_breakpoints == MAX_BREAKPOINTS {
            Err(())
        } else {
            // Scan for the next open slot:
            let mut next_free: usize = 0;

            while let Some(_) = self.breakpoints[next_free] {
                next_free += 1;
            }

            assert!(next_free < MAX_BREAKPOINTS, "Invariant violated.");

            self.breakpoints[next_free] = Some(addr);
            Ok(next_free)
        }
    }

    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        if idx < MAX_BREAKPOINTS {
            self.breakpoints[idx].take().map(|_| ()).ok_or(())
        } else {
            Err(())
        }
    }

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        self.breakpoints
    }

    // TODO: breakpoints and watchpoints look macroable
    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        if self.num_set_watchpoints == MAX_MEMORY_WATCHPOINTS {
            Err(())
        } else {
            // Scan for the next open slot:
            let mut next_free: usize = 0;

            while let Some(_) = self.watchpoints[next_free] {
                next_free += 1;
            }

            assert!(next_free < MAX_MEMORY_WATCHPOINTS, "Invariant violated.");

            self.watchpoints[next_free] = Some((addr, self.read_word(addr)));
            Ok(next_free)
        }
    }

    fn unset_memory_watchpoint(&mut self, idx: usize) -> Result<(), ()> {
        if idx < MAX_MEMORY_WATCHPOINTS {
            self.watchpoints[idx].take().map(|_| ()).ok_or(())
        } else {
            Err(())
        }
    }

    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS] {
        self.watchpoints
    }

    fn run_until_event(&mut self) -> Self::EventFuture {
        // DO NOT IMPLEMENT, yet
        unimplemented!()
    }

    fn step(&mut self) -> State {
        match self.interp.step() {
            MachineState::Running => {
                self.state = State::Paused;
            }
            MachineState::Halted => {
                self.state = State::Halted;
            }
        }

        self.get_state()
    }

    fn pause(&mut self) {
        unimplemented!()
    }

    fn get_state(&self) -> State {
        self.state
    }

    fn reset(&mut self) {
        self.state = State::Paused;
        InstructionInterpreter::reset(&mut self.interp);
        self.shared_state.as_ref().map(|s| s.reset());
    }

    fn get_error(&self) -> Option<Error> {
        unimplemented!()
    }

    fn get_gpio_states(&self) -> GpioPinArr<GpioState> {
        Gpio::get_states(self.interp.get_peripherals())
    }

    fn get_gpio_readings(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        Gpio::read_all(self.interp.get_peripherals())
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState> {
        Adc::get_states(self.interp.get_peripherals())
    }

    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
        Adc::read_all(self.interp.get_peripherals())
    }

    fn get_timer_states(&self) -> TimerArr<TimerState> {
        Timers::get_states(self.interp.get_peripherals())
    }

    fn get_timer_config(&self) -> TimerArr<Word> {
        Timers::get_periods(self.interp.get_peripherals())
    }

    fn get_pwm_states(&self) -> PwmPinArr<PwmState> {
        Pwm::get_states(self.interp.get_peripherals())
    }

    fn get_pwm_config(&self) -> PwmPinArr<u8> {
        Pwm::get_duty_cycles(self.interp.get_peripherals())
    }

    fn get_clock(&self) -> Word {
        Clock::get_milliseconds(self.interp.get_peripherals())
    }
}

// #[derive(Debug)]
// pub struct SimFuture<'S, S: EventFutureSharedStatePorcelain>(&'s S);

// impl<'s> Future for SimFuture<'s> {
//     type Output = (Event, State);

//     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {

//     }
// }
