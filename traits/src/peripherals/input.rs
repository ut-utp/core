//! [`Input` device trait](Input) and related things.
use crate::peripheral_trait;

use core::sync::atomic::AtomicBool;

peripheral_trait! {input,
pub trait Input<'a>: Default {
    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool);
    fn interrupt_occurred(&mut self) -> bool;
    
    fn set_interrupt_enable_bit(&mut self, bit: bool);
    fn interrupts_enabled(&self) -> bool;
    
    // Warning! This is stateful!! It marks the current data as read.
    //
    // Also note: this is technically infallible (it's up to the
    // interpreter what to do for some of the edge cases, but
    // we'll presumably just return some default value) but since
    // we're letting the interpreter decide we *do* return a Result
    // type here.
    fn read_data(&mut self) -> Result<u8, InputError>;
    fn current_data_unread(&mut self) -> bool;
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InputError;

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};

    impl<'a, I: Input<'a>> Input<'a> for Arc<RwLock<I>> {
        fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
            RwLock::write(self).unwrap().register_interrupt_flag(flag)
        }
        
        fn interrupt_occurred(&mut self) -> bool {
            RwLock::write(self).unwrap().interrupt_occurred()
        }
        
        fn set_interrupt_enable_bit(&mut self, bit: bool) {
            RwLock::write(self).unwrap().set_interrupt_enable_bit(bit)
        }
        
        fn interrupts_enabled(&self) -> bool {
            RwLock::read(self).unwrap().interrupts_enabled()
        }
        
        fn read_data(&mut self) -> Result<u8, InputError> {
            RwLock::write(self).unwrap().read_data()
        }
        
        fn current_data_unread(&mut self) -> bool {
            RwLock::write(self).unwrap().current_data_unread()
        }
    }
}
