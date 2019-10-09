//! [`Clock` trait](Clock).

use crate::Word;

// Just 1 Clock! (millisecond units)
pub trait Clock {
    // fn enable(&mut self);   // Probably ditch (TODO).
    // fn disable(&mut self);

    fn get_milliseconds(&self) -> Word;
    fn set_milliseconds(&mut self, ms: Word);
}
