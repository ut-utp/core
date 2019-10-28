//! [`Clock` trait](Clock).

use crate::peripheral_trait;
use crate::Word;

// Just 1 Clock! (millisecond units)
peripheral_trait! {clock,
pub trait Clock: Default {
    // fn enable(&mut self);   // Probably ditch (TODO).
    // fn disable(&mut self);

    fn get_milliseconds(&self) -> Word;
    // isn't milliseconds too large?
    // shouldn't it be more like nano, because PLL can generate
    // frequencies between 3.12MHz to 80MHz - TExaS_Init sets at 80MHz from what I remember
    fn set_milliseconds(&mut self, ns: Word); // want to be able to set to 80MHz, requiring 12.5 nano seconds 
    
}}
