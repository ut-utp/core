use crate::peripherals::OwnedOrRef;

use lc3_traits::peripherals::input::{Input, InputError};

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};
use std::io::{stdin, Read};
use std::sync::Mutex;

/// The source from which Inputs will read characters.
///
/// Generally expected to behave as a one-character buffer holding the latest
/// character input to the peripheral.
pub trait Source {
    /// THIS FUNCTION MUST NOT TAKE SIGNIFICANT TIME (BLOCK).
    /// Returns None if the last character has already been read.
    /// Returns Some(last char input) if this function hasn't previously
    /// returned that input.
    /// If this function isn't called before a new character is input
    /// (which is unlikely, as this function is called every simulator cycle),
    /// only the newest character is expected to be returned
    /// (i.e. this shouldn't behave as a multi-character buffer).
    fn get_char(&self) -> Option<u8>;
}

pub struct SourceShim {
    last_char: Mutex<Option<u8>>,
}

impl SourceShim {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&self, c: char) {
        if c.is_ascii() {
            let mut last_char = self.last_char.lock().unwrap();
            last_char.replace(c as u8);
        } else {
            // TODO: don't ignore non-ASCII
        }
    }
}

impl Default for SourceShim {
    fn default() -> Self {
        Self {
            last_char: Mutex::new(None),
        }
    }
}

impl Source for SourceShim {
    fn get_char(&self) -> Option<u8> {
        let mut last_char = self.last_char.lock().unwrap();
        last_char.take()
    }
}

// #[derive(Clone)] // TODO: Debug
pub struct InputShim<'i, 'int> {
    source: OwnedOrRef<'i, dyn Source + Send + Sync + 'i>,
    flag: Option<&'int AtomicBool>,
    interrupt_enable_bit: bool,
    data: Cell<Option<u8>>,
}

pub struct StdinSource;

// This blocks; don't use this unless you know what you're doing.
//
// We must implement Default because of our tyrannical super trait bounds.
impl Source for StdinSource {
    fn get_char(&self) -> Option<u8> {
        let mut buf: [u8; 1] = [0];

        match stdin().read(&mut buf) {
            Ok(_) => Some(buf[0]),
            Err(_) => None,
        }
    }
}

// By default this reads from something that will never produce new values; this
// is effectively useless.
impl Default for InputShim<'_, '_> {
    fn default() -> Self {
        Self::using(Box::new(SourceShim::new()))
    }
}

impl<'int, 'i> InputShim<'i, 'int> {
    fn new() -> Self {
        Self::default()
    }

    fn sourced_from(source: OwnedOrRef<'i, dyn Source + Send + Sync + 'i>) -> Self {
        Self {
            source,
            interrupt_enable_bit: false,
            flag: None,
            data: Cell::new(None),
        }
    }

    pub fn using(source: Box<dyn Source + Send + Sync + 'i>) -> Self {
        InputShim::sourced_from(OwnedOrRef::Owned(source))
    }

    pub fn with_ref(source: &'i (dyn Source + Send + Sync + 'i)) -> Self {
        InputShim::sourced_from(OwnedOrRef::Ref(source))
    }

    fn fetch_latest(&self) {
        let new_data = self.source.get_char();
        if let Some(c) = new_data {
            self.data.replace(Some(c));
            match self.flag {
                Some(flag) => flag.store(true, Ordering::SeqCst),
                None => unreachable!(),
            }
        }
    }
}

impl<'int: 'i, 'i> Input<'int> for InputShim<'i, 'int> {
    fn register_interrupt_flag(&mut self, flag: &'int AtomicBool) {
        self.flag = match self.flag {
            None => Some(flag),
            Some(_) => {
                // warn!("re-registering interrupt flags!");
                Some(flag)
            }
        }
    }

    fn interrupt_occurred(&self) -> bool {
        self.current_data_unread()
    }

    fn reset_interrupt_flag(&mut self) {
        match self.flag {
            Some(flag) => flag.store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
    }

    fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupt_enable_bit = bit;
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupt_enable_bit
    }

    fn read_data(&self) -> Result<u8, InputError> {
        self.fetch_latest();
        match self.flag {
            Some(flag) => flag.store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
        self.data.get().ok_or(InputError)
    }

    fn current_data_unread(&self) -> bool {
        self.fetch_latest();
        match self.flag {
            Some(flag) => flag.load(Ordering::SeqCst),
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use lc3_test_infrastructure::{assert_eq, assert_ne};
}
