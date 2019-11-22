use crate::interp::{InstructionInterpreter, InstructionInterpreterPeripheralAccess, MachineState};

use lc3_isa::{Addr, Reg, Word};
use lc3_traits::control::{Control, Event, State};
use lc3_traits::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use lc3_traits::error::Error;
use lc3_traits::memory::MemoryMiscError;
use lc3_traits::peripherals::adc::{Adc, AdcPinArr, AdcReadError, AdcState};
use lc3_traits::peripherals::clock::Clock;
use lc3_traits::peripherals::gpio::{Gpio, GpioPinArr, GpioReadError, GpioState};
use lc3_traits::peripherals::pwm::{Pwm, PwmPinArr, PwmState};
use lc3_traits::peripherals::timers::{TimerArr, TimerState, Timers};
use lc3_traits::peripherals::Peripherals;

use core::future::Future;
use core::marker::PhantomData;
use core::ops::Deref;
use core::pin::Pin;
use core::task::{Context, Poll};

struct Simulator<'a, I: InstructionInterpreter + InstructionInterpreterPeripheralAccess<'a>>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    interp: I,
    breakpoints: [Option<Addr>; MAX_BREAKPOINTS],
    watchpoints: [Option<(Addr, Word)>; MAX_MEMORY_WATCHES], // TODO: change to throw these when the location being watched to written to; not just when the value is changed...
    num_set_breakpoints: usize,
    num_set_watchpoints: usize,
    state: State,
    _i: PhantomData<&'a ()>,
}

impl<'a, I: InstructionInterpreterPeripheralAccess<'a> + Default> Default for Simulator<'a, I>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<'a, I: InstructionInterpreterPeripheralAccess<'a>> Simulator<'a, I>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    fn new(interp: I) -> Self {
        Self {
            interp,
            breakpoints: [None; MAX_BREAKPOINTS],
            watchpoints: [None; MAX_MEMORY_WATCHES],
            num_set_breakpoints: 0,
            num_set_watchpoints: 0,
            state: State::Paused,
            _i: PhantomData,
        }
    }
}

impl<'a, I: InstructionInterpreterPeripheralAccess<'a>> Control for Simulator<'a, I>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    type EventFuture = SimFuture;

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
    fn set_memory_watch(&mut self, addr: Addr, data: Word) -> Result<usize, ()> {
        if self.num_set_watchpoints == MAX_MEMORY_WATCHES {
            Err(())
        } else {
            // Scan for the next open slot:
            let mut next_free: usize = 0;

            while let Some(_) = self.watchpoints[next_free] {
                next_free += 1;
            }

            assert!(next_free < MAX_MEMORY_WATCHES, "Invariant violated.");

            self.watchpoints[next_free] = Some((addr, data));
            Ok(next_free)
        }
    }

    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()> {
        if idx < MAX_MEMORY_WATCHES {
            self.watchpoints[idx].take().map(|_| ()).ok_or(())
        } else {
            Err(())
        }
    }

    fn get_memory_watches(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] {
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
        unimplemented!()
    }

    fn get_error(&self) -> Option<Error> {
        unimplemented!()
    }

    fn get_gpio_states(&self) -> GpioPinArr<GpioState> {
        Gpio::get_states(self.interp.get_peripherals())
    }

    fn get_gpio_reading(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        Gpio::read_all(self.interp.get_peripherals())
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState> {
        Adc::get_states(self.interp.get_peripherals())
    }

    fn get_adc_reading(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
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

#[derive(Debug)]
pub struct SimFuture;

impl Future for SimFuture {
    type Output = Event;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}
