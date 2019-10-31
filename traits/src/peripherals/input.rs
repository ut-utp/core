//! [`Input` device trait](Input) and related things.

use crate::peripheral_trait;

peripheral_trait! {input,
pub trait Input: Default {
    /// Read a single ASCII character.
    /// Returns Err if the read fails.
    fn read(&self) -> Result<u8, ReadError>;
    
}}

pub struct ReadError;