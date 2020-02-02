//! Stub implementations of the peripheral traits. Useful for situations in
//! which the peripherals aren't used (or actual functionality isn't desired).

use lc3_isa::Word;
use super::{Gpio, Adc, Pwm, Timers, Clock, Input, Output, PeripheralSet};
use core::sync::atomic::AtomicBool;

pub type PeripheralsStub<'s> = PeripheralSet<
    's,
    GpioStub,
    AdcStub,
    PwmStub,
    TimersStub,
    ClockStub,
    InputStub,
    OutputStub,
>;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct GpioStub;

use super::gpio::{GpioPin, GpioState, GpioMiscError, GpioPinArr, GpioReadError, GpioWriteError};
impl<'a> Gpio<'a> for GpioStub {
    fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError> { Err(GpioMiscError) }
    fn get_state(&self, pin: GpioPin) -> GpioState { GpioState::Disabled }

    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> { Err(GpioReadError((pin, GpioState::Disabled))) }
    fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError> { Err(GpioWriteError((pin, GpioState::Disabled))) }

    fn register_interrupt_flags(&mut self, flags: &'a GpioPinArr<AtomicBool>) {}
    fn interrupt_occurred(&self, pin: GpioPin) -> bool { false }
    fn reset_interrupt_flag(&mut self, pin: GpioPin) { }
    fn interrupts_enabled(&self, pin: GpioPin) -> bool { false }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AdcStub;

use super::adc::{AdcPin, AdcState, AdcPinArr, AdcReadError};
impl Adc for AdcStub {
    fn set_state(&mut self, pin: AdcPin, _: AdcState) -> Result<(), ()> { Err(()) }
    fn get_state(&self, pin: AdcPin) -> AdcState { AdcState::Disabled }

    fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError> { Err(AdcReadError((pin, AdcState::Disabled)))}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PwmStub;

use super::pwm::{PwmPin, PwmState, PwmSetPeriodError, PwmPinArr, PwmSetDutyError};
impl Pwm for PwmStub {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError> { Err(PwmSetPeriodError(pin)) }
    fn get_state(&self, pin: PwmPin) -> PwmState { PwmState::Disabled }

    fn get_pin(&self, pin: PwmPin) -> bool { false }
    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError> { Err(PwmSetDutyError(pin)) }

    fn get_duty_cycle(&self, pin: PwmPin) -> u8 { 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimersStub;

use super::timers::{TimerId, TimerState, TimerMiscError, TimerArr};
impl<'a> Timers<'a> for TimersStub {
    fn set_state(&mut self, timer: TimerId, state: TimerState) -> Result<(), TimerMiscError> { Err(TimerMiscError) }
    fn get_state(&self, timer: TimerId) -> TimerState { TimerState::Disabled }

    fn set_period(&mut self, timer: TimerId, ms: Word) -> Result<(), TimerMiscError> { Err(TimerMiscError) }
    fn get_period(&self, timer: TimerId) -> Word { 0 }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {}
    fn interrupt_occurred(&self, timer: TimerId) -> bool { false }
    fn reset_interrupt_flag(&mut self, timer: TimerId) { }
    fn interrupts_enabled(&self, timer: TimerId) -> bool { false }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClockStub;

impl Clock for ClockStub {
    fn get_milliseconds(&self) -> Word { 0 }
    fn set_milliseconds(&mut self, ms: Word) { }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct InputStub;

use super::input::InputError;
impl<'a> Input<'a> for InputStub {
    fn read_data(&self) -> Result<u8, InputError> { Err(InputError) }
    fn current_data_unread(&self) -> bool { false }

    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) { }
    fn interrupt_occurred(&self) -> bool { false }

    fn set_interrupt_enable_bit(&mut self, bit: bool) { }
    fn interrupts_enabled(&self) -> bool  { false }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OutputStub;

use super::output::OutputError;
impl<'a> Output<'a> for OutputStub {
    fn write_data(&mut self, c: u8) -> Result<(), OutputError> { Ok(()) }
    fn current_data_written(&self) -> bool { true }

    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) { }
    fn interrupt_occurred(&self) -> bool { false }

    fn set_interrupt_enable_bit(&mut self, bit: bool) { }
    fn interrupts_enabled(&self) -> bool { false }
}
