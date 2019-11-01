//! [`Input` device trait](Input) and related things.
use crate::peripheral_trait;

peripheral_trait! {input,
pub trait Input: Default {
    /// Read a single ASCII character.
    /// Returns Err if the read fails.
    fn read(&mut self) -> Result<u8, ReadError>; // TODO: I think this should maybe go away..

    // fn register_interrupt(&mut self, ) // TODO!!!
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReadError;

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<I: Input> Input for Arc<RwLock<I>> {
        fn read(&mut self) -> Result<u8, ReadError> {
            RwLock::write(self).unwrap().read()
        }
    }

    // don't do this:
    // use std::ops::Deref;
    // impl<I: Input, Z: Default + Deref<Target = I>> Input for Z {
    //     fn read(&mut self) -> Result<u8, ReadError> {
    //         self.read()
    //     }
    // }
}
