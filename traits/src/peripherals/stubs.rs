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
    fn set_state(&mut self, _pin: GpioPin, _state: GpioState) -> Result<(), GpioMiscError> { Err(GpioMiscError) }
    fn get_state(&self, _pin: GpioPin) -> GpioState { GpioState::Disabled }

    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> { Err(GpioReadError((pin, GpioState::Disabled))) }
    fn write(&mut self, pin: GpioPin, _bit: bool) -> Result<(), GpioWriteError> { Err(GpioWriteError((pin, GpioState::Disabled))) }

    fn register_interrupt_flags(&mut self, _flags: &'a GpioPinArr<AtomicBool>) {}
    fn interrupt_occurred(&self, _pin: GpioPin) -> bool { false }
    fn reset_interrupt_flag(&mut self, _pin: GpioPin) { }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AdcStub;

use super::adc::{AdcPin, AdcState, AdcPinArr, AdcReadError, AdcMiscError};
impl Adc for AdcStub {
    fn set_state(&mut self, _pin: AdcPin, _: AdcState) -> Result<(), AdcMiscError> { Err(AdcMiscError) }
    fn get_state(&self, _pin: AdcPin) -> AdcState { AdcState::Disabled }

    fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError> { Err(AdcReadError((pin, AdcState::Disabled)))}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PwmStub;

use super::pwm::{PwmPin, PwmState, PwmPinArr, PwmDutyCycle};
impl Pwm for PwmStub {
    fn set_state(&mut self, _pin: PwmPin, _state: PwmState) { }
    fn get_state(&self, _pin: PwmPin) -> PwmState { PwmState::Disabled }

    fn set_duty_cycle(&mut self, _pin: PwmPin, _duty: PwmDutyCycle) { }

    fn get_duty_cycle(&self, _pin: PwmPin) -> PwmDutyCycle { 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TimersStub;

use super::timers::{TimerId, TimerArr, TimerMode, TimerState};
impl<'a> Timers<'a> for TimersStub {
    fn set_mode(&mut self, _timer: TimerId, _mode: TimerMode) { }
    fn get_mode(&self, _timer: TimerId) -> TimerMode { TimerMode::SingleShot }

    fn set_state(&mut self, _timer: TimerId, _state: TimerState) { }
    fn get_state(&self, _timer: TimerId) -> TimerState { TimerState::Disabled }

    fn register_interrupt_flags(&mut self, _flags: &'a TimerArr<AtomicBool>) {}
    fn interrupt_occurred(&self, _timer: TimerId) -> bool { false }
    fn reset_interrupt_flag(&mut self, _timer: TimerId) { }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClockStub;

impl Clock for ClockStub {
    fn get_milliseconds(&self) -> Word { 0 }
    fn set_milliseconds(&mut self, _ms: Word) { }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct InputStub;

use super::input::InputError;
impl<'a> Input<'a> for InputStub {
    fn read_data(&self) -> Result<u8, InputError> { Err(InputError) }
    fn current_data_unread(&self) -> bool { false }

    fn register_interrupt_flag(&mut self, _flag: &'a AtomicBool) { }
    fn interrupt_occurred(&self) -> bool { false }
    fn reset_interrupt_flag(&mut self) { }

    fn set_interrupt_enable_bit(&mut self, _bit: bool) { }
    fn interrupts_enabled(&self) -> bool  { false }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OutputStub;

use super::output::OutputError;

impl<'a> Output<'a> for OutputStub {
    fn write_data(&mut self, c: u8) -> Result<(), OutputError> { Ok(()) }
    fn current_data_written(&self) -> bool { true }

    fn register_interrupt_flag(&mut self, _flag: &'a AtomicBool) { }
    fn interrupt_occurred(&self) -> bool { false }
    fn reset_interrupt_flag(&mut self) { }

    fn set_interrupt_enable_bit(&mut self, _bit: bool) { }
    fn interrupts_enabled(&self) -> bool { false }
}
