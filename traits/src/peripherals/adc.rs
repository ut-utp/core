//! [`Adc` trait](Adc) and associated types.

use crate::peripheral_trait;
use core::ops::{Deref, Index, IndexMut};

// TODO: Add Errors

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdcPin { A0, A1, A2, A3 }

impl AdcPin {
    pub const NUM_PINS: usize = 4;
}

pub const ADC_PINS: AdcPinArr<AdcPin> = {
    use AdcPin::*;
    AdcPinArr([A0, A1, A2, A3])
}; // TODO: once we get the derive macro, get rid of this.

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdcState {
    Enabled,
    Disabled,
    Interrupt,
}

impl From<AdcPin> for usize {
    fn from(pin: AdcPin) -> usize {
        use AdcPin::*;
        match pin {
            A0 => 0,
            A1 => 1,
            A2 => 2,
            A3 => 3,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AdcPinArr<T>(pub [T; AdcPin::NUM_PINS]);

impl<T> Deref for AdcPinArr<T> {
    type Target = [T; AdcPin::NUM_PINS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<AdcPin> for AdcPinArr<T> {
    type Output = T;

    fn index(&self, pin: AdcPin) -> &Self::Output {
        &self.0[usize::from(pin)]
    }
}

impl<T> IndexMut<AdcPin> for AdcPinArr<T> {
    fn index_mut(&mut self, pin: AdcPin) -> &mut Self::Output {
        &mut self.0[usize::from(pin)]
    }
}

pub type AdcHandler<'a> = &'a (dyn Fn(AdcPin, u8) + Sync);

peripheral_trait! {adc,

/// Adc access for the interpreter.
pub trait Adc<'a>: Default {
    fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), ()>;
    fn get_state(&self, pin: AdcPin) -> AdcState;
    fn get_states(&self) -> AdcPinArr<AdcState> {
        let mut states = AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]);

        ADC_PINS
            .iter()
            .for_each(|a| states[*a] = self.get_state(*a));

        states
    }

    fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError>;
    fn read_all(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
        // TODO: Error conversion impl (see Gpio)
        let mut readings = AdcPinArr([Ok(0u8); AdcPin::NUM_PINS]); // TODO: that we need a default value here is weird and bad...

        ADC_PINS
            .iter()
            .for_each(|a| readings[*a] = self.read(*a));

        readings
    }

    fn register_interrupt(
        &mut self,
        pin: AdcPin,
        handler: AdcHandler<'a>
    ) -> Result<(), AdcMiscError>;
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AdcMiscError;

pub type AdcStateMismatch = (AdcPin, AdcState);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AdcReadError(pub AdcStateMismatch);


// TODO: Into Error stuff (see Gpio)

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<'a, A: Adc<'a>> Adc<'a> for Arc<RwLock<A>> {
        fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), ()> {
            RwLock::write(self).unwrap().set_state(pin, state)
        }

        fn get_state(&self, pin: AdcPin) -> AdcState {
            RwLock::read(self).unwrap().get_state(pin)
        }

        fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError> {
            RwLock::read(self).unwrap().read(pin)
        }

        fn register_interrupt(
            &mut self,
            pin: AdcPin,
            handler: AdcHandler<'a>,
        ) -> Result<(), AdcMiscError> {
            RwLock::write(self)
                .unwrap()
                .register_interrupt(pin, handler)
        }
    }
}
