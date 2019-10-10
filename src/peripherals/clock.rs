//! [`Clock` trait](Clock).

use crate::Word;
use crate::peripheral_trait;


// trace_macros!(true);
// trait foo {
//     fn yak(crate::self_macro!(self)) -> ();
// }

// Just 1 Clock! (millisecond units)
peripheral_trait! {clock,
pub trait Clock {
    // fn enable(&mut self);   // Probably ditch (TODO).
    // fn disable(&mut self);

    fn get_milliseconds(&self) -> Word;
    fn set_milliseconds(&mut self, ms: Word);
}}
