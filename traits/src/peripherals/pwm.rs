//! [`Pwm` trait](Pwm) and helpers.

use crate::peripheral_trait;

use lc3_macros::DisplayUsingDebug;

use core::num::NonZeroU8;
use core::ops::{Deref, Index, IndexMut};

use serde::{Deserialize, Serialize};
// TODO: Switch to enum for pins
// TODO: Add Errors
#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum PwmPin { P0, P1 }

// TODO: remove once the derive macro happens...
impl PwmPin {
    pub const NUM_PINS: usize = 2; // P0 - P1
}

impl From<PwmPin> for usize {
    fn from(pin: PwmPin) -> usize {
        use PwmPin::*;
        match pin {
            P0 => 0,
            P1 => 1,
        }
    }
}

pub const PWM_PINS: PwmPinArr<PwmPin> = {
    use PwmPin::*;
    PwmPinArr([P0, P1])
}; // TODO: save us, derive macro

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PwmState {
    Enabled(NonZeroU8),
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PwmPinArr<T>(pub [T; PwmPin::NUM_PINS]);

// Once const fn is more stable:
// impl<T: Copy> PwmPinArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; PwmPin::NUM_PINS])
//     }
// }

impl<T> Deref for PwmPinArr<T> {
    type Target = [T; PwmPin::NUM_PINS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<PwmPin> for PwmPinArr<T> {
    type Output = T;

    fn index(&self, pin: PwmPin) -> &Self::Output {
        &self.0[usize::from(pin)]
    }
}

impl<T> IndexMut<PwmPin> for PwmPinArr<T> {
    fn index_mut(&mut self, pin: PwmPin) -> &mut Self::Output {
        &mut self.0[usize::from(pin)]
    }
}

// I have no idea why these operations wouldn't be infallible, tbh:
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PwmSetPeriodError(pub PwmPin); // TODO: review

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PwmSetDutyError(pub PwmPin); // TODO: review

peripheral_trait! {pwm,
pub trait Pwm: Default {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError>;
    fn get_state(&self, pin: PwmPin) -> PwmState;
    fn get_states(&self) -> PwmPinArr<PwmState> {
        let mut states = PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]);

        PWM_PINS
            .iter()
            .for_each(|p| states[*p] = self.get_state(*p));

        states
    }
    fn get_pin(&self, pin: PwmPin) -> bool; // TODO: should perhaps not be infallible (actually why does this even exist?)
    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError>;

    // TODO: why is this infallible?
    fn get_duty_cycle(&self, pin: PwmPin) -> u8; // This is u8 because u16 fractions seem excessive.
    fn get_duty_cycles(&self) -> PwmPinArr<u8> {
        let mut duty_cycles = PwmPinArr([0u8; PwmPin::NUM_PINS]);

        PWM_PINS
            .iter()
            .for_each(|p| duty_cycles[*p] = self.get_duty_cycle(*p));

        duty_cycles
    }
}}

// TODO: Into Error stuff (see Gpio)

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<P: Pwm> Pwm for Arc<RwLock<P>> {
        fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError> {
            RwLock::write(self).unwrap().set_state(pin, state)
        }

        fn get_state(&self, pin: PwmPin) -> PwmState {
            RwLock::read(self).unwrap().get_state(pin)
        }
        fn get_pin(&self, pin: PwmPin) -> bool {
            RwLock::read(self).unwrap().get_pin(pin)
        }

        fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError> {
            RwLock::write(self).unwrap().set_duty_cycle(pin, duty)
        }

        fn get_duty_cycle(&self, pin: PwmPin) -> u8 {
            RwLock::read(self).unwrap().get_duty_cycle(pin)
        }
    }
}
