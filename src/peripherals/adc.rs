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
// trace_macros!(true);

peripheral_trait! {adc,
pub trait Adc {
    fn set_state(&mut self, pin: u8, state: AdcState) -> Result<(), ()>;
    fn get_state(&self, pin: u8) -> Option<AdcState>;
    // fn get_states() // TODO


    fn read(&self, pin: u8) -> Result<u8, ()>;

    fn register_interrupt(&mut self, pin: u8, func: impl FnMut(u8)) -> Result<(), ()>;
}}

// use crate::peripheral_trait;

// peripheral_trait!(adc,
// pub trait Adc {
//     fn set_state(&mut self, pin: u8, state: AdcState) -> Result<(), ()>;
//     fn get_state(&self, pin: u8) -> Option<AdcState>;
//     // fn get_states() // TODO

//     fn read(&self, pin: u8) -> Result<u8, ()>;

//     fn register_interrupt(&mut self, pin: u8, func: impl FnMut(u8)) -> Result<(), ()>;
// });

// use crate::peripheral_set_impl;

// Impl for PeripheralSet
// peripheral_set_impl!(Adc, {
//     fn set_state(&mut self, pin: u8, state: AdcState) -> Result<(), ()> {
//         self.adc.set_state(pin, state)
//     }
//     fn get_state(&self, pin: u8) -> Option<AdcState> { self.adc.get_state(pin) }

//     fn read(&self, pin: u8) -> Result<u8, ()> { self.adc.read(pin) }

//     fn register_interrupt(&mut self, pin: u8, func: impl FnMut(u8)) -> Result<(), ()> {
//         self.adc.register_interrupt(pin, func)
//     }
// });
