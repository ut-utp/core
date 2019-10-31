//! [`Pwm` trait](Pwm) and helpers.

use crate::peripheral_trait;
use core::num::NonZeroU8;
// TODO: Switch to enum for pins
// TODO: Add Errors
#[rustfmt::skip]
#[derive(Copy, Clone)]
pub enum PwmPin { P0, P1 }
pub const NUM_PWM_PINS: u8 = 2; // P0 - P1
pub enum PwmState {
    Enabled(NonZeroU8),
    Disabled,
}
pub type PwmPinArr<T> = [T; NUM_PWM_PINS as usize];

impl From<PwmPin> for usize {
    fn from(pin: PwmPin) -> usize {
        use PwmPin::*;
        match pin {
            P0 => 0,
            P1 => 1,
        }
    }
}

peripheral_trait! {pwm,
pub trait Pwm: Default {
    // enable, disable, set duty cycle, enable hystersis. start
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), ()>;
    fn get_state(&self, pin: PwmPin) -> Option<PwmState>;
    // fn get_states() // TODO
   // fn set_period(&mut self, pin:PwmPin); 
    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8); // TODO: made mutable, review
   
    //fn start(&mut self, pin: PwmPin); // Start the periodic timer interrupt
    //fn disable(&mut self, pin: PwmPin);
}}
