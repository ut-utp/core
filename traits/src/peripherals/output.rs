//! [`Output` device trait](Output) and friends.

use crate::peripheral_trait;

peripheral_trait! {output,
pub trait Output: Default {

    fn write(&self, c: u8) -> Result<(), WriteError>;    

}}

pub struct WriteError;
