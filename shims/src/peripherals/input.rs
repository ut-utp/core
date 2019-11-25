use core::sync::atomic::{AtomicBool, Ordering};
use lc3_traits::peripherals::input::{Input, InputError};
use std::io::{stdin, Read};
use std::sync::Mutex;

/// The source from which Inputs will read characters.
/// Generally expected to behave as a one-character buffer holding the latest
/// character input to the peripheral. 
pub trait Source: Default {
    /// THIS FUNCTION MUST NOT TAKE SIGNIFICANT TIME (BLOCK).
    /// Returns None if the last character has already been read.
    /// Returns Some(last char input) if this function hasn't previously returned that input.
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
    fn new() -> Self {
        Self::default()
    }
    fn push(&self, c: char) {
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
            last_char: Mutex::new(None)
        }
    }
}

impl Source for SourceShim {
    fn get_char(&self) -> Option<u8> {
        let mut last_char = self.last_char.lock().unwrap();
        last_char.take()
    }
}

pub struct InputShim<'a, S: Source> {
    source: S,
    flag: Option<&'a AtomicBool>,
    interrupt_enable_bit: bool,
    data: Option<u8>,
}

impl<S: Source> Default for InputShim<'_, S> {
    fn default() -> Self {
        Self::sourced_from(S::default())
    }
}

impl<S: Source> InputShim<'_, S> {
    fn new() -> Self {
        Self::default()
    }

    fn sourced_from(source: S) -> Self {
        Self {
            interrupt_enable_bit: false,
            source,
            flag: None,
            data: None,
        }
    }
    
    fn fetch_latest(&mut self) {
        let new_data = self.source.get_char();
        if let Some(char) = new_data {
            self.data = Some(char);
            match self.flag {
                Some(flag) => flag.store(true, Ordering::SeqCst),
                None => unreachable!(),
            }
        }
    }
}

impl<'a, S: Source> Input<'a> for InputShim<'a, S> {
    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
        self.flag = match self.flag {
            None => Some(flag),
            Some(_) => unreachable!(),
        }
    }

    fn interrupt_occurred(&mut self) -> bool {
        self.current_data_unread()
    }

    fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupt_enable_bit = bit;
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupt_enable_bit
    }

    fn read_data(&mut self) -> Result<u8, InputError> {
        self.fetch_latest();
        match self.flag {
            Some(flag) => flag.store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
        self.data.ok_or(InputError)
    }

    fn current_data_unread(&mut self) -> bool {
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
}
