//! [`Adc` trait](Adc) and associated types.

use crate::peripheral_trait;

// TODO: Switch to enum for pins
// TODO: Add Errors
const NUM_ADC_PINS: u8 = 4; // A0 - A3
pub enum AdcState {
    Enabled,
    Disabled,
    Interrupt,
}

peripheral_trait! {adc,

/// Adc access for the interpreter.
pub trait Adc<'a>: Default {
    fn set_state(&mut self, pin: u8, state: AdcState) -> Result<(), ()>;
    fn get_state(&self, pin: u8) -> Option<AdcState>;
    // fn get_states() // TODO


    fn read(&self, pin: u8) -> Result<u8, ()>;

    fn register_interrupt(&mut self, pin: u8, func: &'a dyn FnMut(u8)) -> Result<(), ()>;
}}
