//! [`Output` device trait](Output) and friends.
use crate::peripheral_trait;

use core::sync::atomic::AtomicBool;

peripheral_trait! {output,
pub trait Output<'a>: Default {
    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool);
    fn interrupt_occurred(&self) -> bool;

    fn set_interrupt_enable_bit(&mut self, bit: bool);
    fn interrupts_enabled(&self) -> bool;

    fn write_data(&mut self, c: u8) -> Result<(), OutputError>;

    // Gets set to high automagically when more data can be taken.
    // Gets set to low (by [write_data](Output::write_data)) when
    // data is being written.
    fn current_data_written(&self) -> bool;
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OutputError;

using_std! {
    use std::sync::{Arc, RwLock};

    impl<'a, O: Output<'a>> Output<'a> for Arc<RwLock<O>> {

        fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
            RwLock::write(self).unwrap().register_interrupt_flag(flag)
        }

        fn interrupt_occurred(&self) -> bool {
            RwLock::write(self).unwrap().interrupt_occurred()
        }

        fn set_interrupt_enable_bit(&mut self, bit: bool) {
            RwLock::write(self).unwrap().set_interrupt_enable_bit(bit)
        }

        fn interrupts_enabled(&self) -> bool {
            RwLock::read(self).unwrap().interrupts_enabled()
        }

        fn write_data(&mut self, c: u8) -> Result<(), OutputError> {
            RwLock::write(self).unwrap().write_data(c)
        }

        fn current_data_written(&self) -> bool {
            RwLock::write(self).unwrap().current_data_written()
        }
    }

}
