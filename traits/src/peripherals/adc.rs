//! [`Adc` trait](Adc) and associated types.

use crate::peripheral_trait;
use core::ops::{ Index, IndexMut };

// TODO: Add Errors

#[rustfmt::skip]
#[derive(Copy, Clone)]
pub enum Pin { A0, A1, A2, A3 }

pub const NUM_PINS: u8 = 4;
pub struct PinArr<T>(pub [T; NUM_PINS as usize]);

#[derive(Copy, Clone)]
pub enum State {
    Enabled,
    Disabled,
    Interrupt,
}

impl From<Pin> for usize {
    fn from(pin: Pin) -> usize {
        use Pin::*;
        match pin {
            A0 => 0,
            A1 => 1,
            A2 => 2,
            A3 => 3,
        }
    }
}

impl<T> Index<Pin> for PinArr<T> {
    type Output = T;

    fn index(&self, pin: Pin) -> &Self::Output {
        &self.0[usize::from(pin)]
    }
}

impl<T> IndexMut<Pin> for PinArr<T> {
    fn index_mut(&mut self, pin: Pin) -> &mut Self::Output {
        &mut self.0[usize::from(pin)]
    }
}

pub type Handler = dyn Fn(u8);

peripheral_trait! {adc,

/// Adc access for the interpreter.
pub trait Adc<'a>: Default {
    fn set_state(&mut self, pin: Pin, state: State) -> Result<(), ()>;
    fn get_state(&self, pin: Pin) -> State;
    // fn get_states() // TODO

    fn read(&self, pin: Pin) -> Result<u8, ReadError>;

    fn register_interrupt(&mut self, pin: Pin, handler: &'a Handler) -> Result<(), ()>;
}}

pub type StateMismatch = (Pin, State);
pub struct ReadError(pub StateMismatch);

