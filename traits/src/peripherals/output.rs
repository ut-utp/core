//! [`Output` device trait](Output) and friends.
use crate::peripheral_trait;

use core::sync::atomic::AtomicBool;
use core::fmt::{self, Display};

use serde::{Deserialize, Serialize};

peripheral_trait! {output,
pub trait Output<'a>: Default {
    fn write_data(&mut self, c: u8) -> Result<(), OutputError>;

    // Gets set to high automagically when more data can be taken.
    // Gets set to low (by [write_data](Output::write_data)) when
    // data is being written.
    fn current_data_written(&self) -> bool;

    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool);
    fn interrupt_occurred(&self) -> bool;
    fn reset_interrupt_flag(&mut self,);

    fn set_interrupt_enable_bit(&mut self, bit: bool);
    fn interrupts_enabled(&self) -> bool;
}}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputError {
    NonUnicodeCharacter(u8),
    IoError,
}

impl Display for OutputError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OutputError::*;

        match self {
            NonUnicodeCharacter(c) => write!(fmt, "Tried to write a non-unicode output: {:#2X}", c),
            IoError => write!(fmt, "I/O error when writing output"),
        }
    }
}

using_std! {
    use std::sync::{Arc, RwLock};
    impl<'a, O: Output<'a>> Output<'a> for Arc<RwLock<O>> {
        fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
            RwLock::write(self).unwrap().register_interrupt_flag(flag)
        }

        fn interrupt_occurred(&self) -> bool {
            RwLock::write(self).unwrap().interrupt_occurred()
        }

        fn reset_interrupt_flag(&mut self) {
            RwLock::write(self).unwrap().reset_interrupt_flag()
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

    use std::sync::Mutex;
    impl<'a, O: Output<'a>> Output<'a> for Arc<Mutex<O>> {
        fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
            Mutex::lock(self).unwrap().register_interrupt_flag(flag)
        }

        fn interrupt_occurred(&self) -> bool {
            Mutex::lock(self).unwrap().interrupt_occurred()
        }

        fn reset_interrupt_flag(&mut self) {
            Mutex::lock(self).unwrap().reset_interrupt_flag()
        }

        fn set_interrupt_enable_bit(&mut self, bit: bool) {
            Mutex::lock(self).unwrap().set_interrupt_enable_bit(bit)
        }

        fn interrupts_enabled(&self) -> bool {
            Mutex::lock(self).unwrap().interrupts_enabled()
        }

        fn write_data(&mut self, c: u8) -> Result<(), OutputError> {
            Mutex::lock(self).unwrap().write_data(c)
        }

        fn current_data_written(&self) -> bool {
            Mutex::lock(self).unwrap().current_data_written()
        }
    }
}
