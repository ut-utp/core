//! [`Input` device trait](Input) and related things.
use crate::peripheral_trait;

use core::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
peripheral_trait! {input,
pub trait Input<'a>: Default {
    /// Read a single ASCII character.
    /// Returns Err if the read fails.
    fn read(&mut self) -> Result<u8, ReadError>; // TODO: I think this should maybe go away..

    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool);
    fn interrupt_occurred(&self) -> bool;
    fn reset_interrupt_flag(&mut self,);
    fn interrupts_enabled(&self) -> bool;
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReadError;

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    use core::sync::atomic::AtomicBool;
    
    impl<'a, I: Input<'a>> Input<'a> for Arc<RwLock<I>> {
        fn read(&mut self) -> Result<u8, ReadError> {
            RwLock::write(self).unwrap().read()
        }
        
        fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
            RwLock::write(self).unwrap().register_interrupt_flag(flag)
        }
        
        fn interrupt_occurred(&self) -> bool {
            RwLock::read(self).unwrap().interrupt_occurred()
        }
        
        fn reset_interrupt_flag(&mut self) {
            RwLock::write(self).unwrap().reset_interrupt_flag()
        }

        fn interrupts_enabled(&self) -> bool {
            RwLock::read(self).unwrap().interrupts_enabled()
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
