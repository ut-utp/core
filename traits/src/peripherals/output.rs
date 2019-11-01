//! [`Output` device trait](Output) and friends.
use crate::peripheral_trait;

peripheral_trait! {output,
pub trait Output: Default {
    /// Write a single ASCII char.
    /// Returns Err if the write fails.
    fn write(&mut self, c: u8) -> Result<(), OutputError>;
}}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OutputError;

using_std! {
    use std::sync::{Arc, RwLock};
    impl<O: Output> Output for Arc<RwLock<O>> {
        fn write(&mut self, c: u8) -> Result<(), OutputError> {
            RwLock::write(self).unwrap().write(c)
        }
    }
}
