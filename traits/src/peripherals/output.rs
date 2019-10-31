//! [`Output` device trait](Output) and friends.
use crate::peripheral_trait;

peripheral_trait! {output,
pub trait Output: Default {
    /// Write a single ASCII char.
    /// Returns Err if the write fails. 
    fn write(&mut self, c: u8) -> Result<(), WriteError>;    

}}

#[derive(Debug, PartialEq)]
pub struct WriteError;
