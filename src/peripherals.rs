use super::{Addr, Word};

trait Peripherals: Gpio + Adc + Pwm + Timers {
    fn generic_init(&mut self);
}

pub const NUM_GPIO_PINS: u8 = 8; // G0 - G7
pub enum GpioState { Input, Output, Interrupt, Disabled }
pub trait Gpio {
    fn change_state(&mut self, pin: u8, state: GpioState) -> Result<(), ()>;
    fn get_state(&self, pin: u8) -> Option<GpioState>;

    fn read(&self, pin: u8) -> Result<bool, ()>;
    fn read_all(&self, pin: u8) -> Result<[bool; NUM_GPIO_PINS as usize], ()>;

    fn write(&mut self, pin: u8, bit: bool) -> Result<(), ()>;
    fn write_all(&mut self, pin: u8, bits: [bool; NUM_GPIO_PINS as usize]) -> Result<(), ()>;

    fn register_interrupt(&mut self, pin: u8, func: impl FnMut(bool)) -> Result<(), ()>;
}

pub const NUM_ADC_PINS: u8 = 4; // A0 - A3
pub enum AdcState { Enabled, Disabled, Interrupt }
trait Adc {
    fn change_state(&mut self, pin: u8, state: AdcState) -> Result<(), ()>;
    fn get_state(&self, pin: u8) -> Option<AdcState>;

    fn read(&self, pin: u8) -> Result<u8, ()>;

    fn register_interrupt(&mut self, pin: u8, func: impl FnMut(u8)) -> Result<(), ()>;
}

pub const NUM_PWM_PINS: u8 = 2; // P0 - A1
trait Pwm {
// enable, disable, set duty cycle, enable hystersis. start
    fn enable(&mut self, pin: u8);
    fn set_duty_cycle(duty: u16);
    //Optionally enable hysterisis ?
    fn start(&mut self, pin: u8); //Start the periodic timer interrupt
    fn disable(&mut self, pin: u8);

}

pub const NUM_TIMERS: u8 = 2; // T0 - T1
pub enum TimerState { Repeated, SingleShot, Disabled }

// Timer periods: [0, core::u16::MAX)
trait Timers {
    fn change_state(&mut self, num: u8, state: TimerState) -> Result<(), ()>;
    fn get_state(&mut self, num: u8) -> Option<TimerState>;

    fn set_period(&mut self, num: u8, milliseconds: Word);
    fn get_period(&mut self, num: u8) -> Option<Word>;

    fn register_interrupt(&mut self, num: u8, func: impl FnMut(u8)) -> Result<(), ()>;
}

// Just 1 Clock! (millisecond units)
trait Clock {
    fn enable(&mut self);
    fn disable(&mut self);

    fn get_milliseconds(&self) -> Word;
    fn set_milliseconds(&mut self, ms: Word);
}
