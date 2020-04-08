//! [`Adc` trait](Adc) and associated types.

use crate::peripheral_trait;

use lc3_macros::DisplayUsingDebug;

use core::convert::TryFrom;
use core::ops::{Deref, Index, IndexMut};

use serde::{Deserialize, Serialize};
// TODO: Add Errors

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum AdcPin { A0, A1, A2, A3, A4, A5 }

impl AdcPin {
    pub const NUM_PINS: usize = 6;
}

pub const ADC_PINS: AdcPinArr<AdcPin> = {
    use AdcPin::*;
    AdcPinArr([A0, A1, A2, A3, A4, A5])
}; // TODO: once we get the derive macro, get rid of this.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdcState {
    Enabled,
    Disabled,
}

impl From<AdcPin> for usize {
    fn from(pin: AdcPin) -> usize {
        use AdcPin::*;
        match pin {
            A0 => 0,
            A1 => 1,
            A2 => 2,
            A3 => 3,
            A4 => 4,
            A5 => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcPinArr<T>(pub [T; AdcPin::NUM_PINS]);

// Once const fn is more stable:
// impl<T: Copy> AdcPinArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; AdcPin::NUM_PINS])
//     }
// }

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

peripheral_trait! {adc,

/// Adc access for the interpreter.
pub trait Adc: Default {
    fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), ()>;
    fn get_state(&self, pin: AdcPin) -> AdcState;
    #[inline]
    fn get_states(&self) -> AdcPinArr<AdcState> {
        let mut states = AdcPinArr([AdcState::Disabled; AdcPin::NUM_PINS]);

        ADC_PINS
            .iter()
            .for_each(|a| states[*a] = self.get_state(*a));

        states
    }

    fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError>;
    #[inline]
    fn read_all(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
        // TODO: Error conversion impl (see Gpio)
        let mut readings = AdcPinArr([Ok(0u8); AdcPin::NUM_PINS]); // TODO: that we need a default value here is weird and bad...

        ADC_PINS
            .iter()
            .for_each(|a| readings[*a] = self.read(*a));

        readings
    }

}}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcMiscError;

pub type AdcStateMismatch = (AdcPin, AdcState);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcReadError(pub AdcStateMismatch);

pub type AdcStateMismatches = AdcPinArr<Option<AdcStateMismatch>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdcReadErrors(pub AdcStateMismatches);

impl TryFrom<AdcPinArr<Result<u8, AdcReadError>>> for AdcReadErrors {
    type Error = ();

    fn try_from(read_errors: AdcPinArr<Result<u8, AdcReadError>>) -> Result<Self, Self::Error> {
        let mut errors: AdcStateMismatches = AdcPinArr([None; AdcPin::NUM_PINS]);

        read_errors
            .iter()
            .enumerate()
            .filter_map(|(idx, res)| {
                res.map_err(|adc_read_error| (idx, adc_read_error)).err()
            })
            .for_each(|(idx, adc_read_error)| {
                errors.0[idx] = Some(adc_read_error.0);
            });

        Ok(AdcReadErrors(errors))
    }
}

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<A: Adc> Adc for Arc<RwLock<A>> {
        fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), ()> {
            RwLock::write(self).unwrap().set_state(pin, state)
        }

        fn get_state(&self, pin: AdcPin) -> AdcState {
            RwLock::read(self).unwrap().get_state(pin)
        }

        fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError> {
            RwLock::read(self).unwrap().read(pin)
        }
    }
}
