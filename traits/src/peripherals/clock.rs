//! [`Clock` trait](Clock).

use crate::peripheral_trait;
use lc3_isa::Word;

// Just 1 Clock! (millisecond units)
//
// TODO: we've just limited ourselves to only being able to count ~65 and a
// half seconds with our fancy clock (2 ^ 16 milliseconds).
//
// We can either have our clock have two words, have two clocks, or switch to
// centiseconds or something (i.e. lower our precision).
//
// We probably want to introduce another word for the number of seconds or
// something. We'll revisit this (TODO).
peripheral_trait! {clock,
pub trait Clock: Default {
    fn get_milliseconds(&self) -> Word;

    fn set_milliseconds(&mut self, ms: Word);
}}

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<C: Clock> Clock for Arc<RwLock<C>> {
        fn get_milliseconds(&self) -> Word {
            RwLock::read(self).unwrap().get_milliseconds()
        }

        fn set_milliseconds(&mut self, ms: Word) {
            RwLock::write(self).unwrap().set_milliseconds(ms)
        }
    }
}
