//! [`Clock` trait](Clock).

use crate::peripheral_trait;
use crate::Word;

// Just 1 Clock! (millisecond units)
peripheral_trait! {clock,
pub trait Clock: Default {
    // fn enable(&mut self);   // Probably ditch (TODO).
    // fn disable(&mut self);

    fn get_milliseconds(&self) -> Word;
    fn set_milliseconds(&mut self, ms: Word);
}}
