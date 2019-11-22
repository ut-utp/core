//! [`Output` device trait](Output) and friends.
use crate::peripheral_trait;

use core::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

peripheral_trait! {output,
pub trait Output<'a>: Default {
    /// Write a single ASCII char.
    /// Returns Err if the write fails.
    fn write(&mut self, c: u8) -> Result<(), OutputError>;

    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool);
    fn interrupt_occurred(&self) -> bool;
    fn reset_interrupt_flag(&mut self,);
    fn interrupts_enabled(&self) -> bool;
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OutputError;

using_std! {
    use std::sync::{Arc, RwLock};

    impl<'a, O: Output<'a>> Output<'a> for Arc<RwLock<O>> {
        fn write(&mut self, c: u8) -> Result<(), OutputError> {
            RwLock::write(self).unwrap().write(c)
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
}
