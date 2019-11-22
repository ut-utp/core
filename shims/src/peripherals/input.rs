use core::sync::atomic::{AtomicBool, Ordering};
use lc3_traits::peripherals::input::{Input, ReadError};
use std::io::{stdin, Read};

pub struct InputShim<'a> {
    interrupts_enabled: bool,
    source: Box<dyn Read>,
    flag: Option<&'a AtomicBool>,
}

impl Default for InputShim<'_> {
    fn default() -> Self {
        Self {
            interrupts_enabled: false,
            source: Box::new(stdin()),
            flag: None,
        }
    }
}

impl InputShim<'_> {
    fn new() -> Self {
        Self::default()
    }

    fn using(source: Box<dyn Read>) -> Self {
        Self {
            interrupts_enabled: false,
            source,
            flag: None,
        }
    }
}

impl<'a> Input<'a> for InputShim<'a> {
    fn read(&mut self) -> Result<u8, ReadError> {
        let mut buf: [u8; 1] = [0];
        match self.source.read(&mut buf) {
            Ok(0) => Err(ReadError),
            Ok(_) => Ok(buf[0]),
            Err(_) => Err(ReadError),
        }
    }

    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
        self.flag = match self.flag {
            None => Some(flag),
            Some(_) => unreachable!(),
        }
    }

    fn interrupt_occurred(&self) -> bool {
        match self.flag {
            Some(f) => {
                let occurred = f.load(Ordering::SeqCst);
                self.interrupts_enabled() && occurred
            }
            None => unreachable!(),
        }
    }

    fn reset_interrupt_flag(&mut self) {
        match self.flag {
            Some(f) => f.store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupts_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_nothing() {
        let source: Box<dyn Read> = Box::new("".as_bytes());
        let mut shim = InputShim::using(source);
        let res = shim.read();
        assert_eq!(res, Err(ReadError));
    }

    #[test]
    fn reads_one() {
        let source: Box<dyn Read> = Box::new("A".as_bytes());
        let mut shim = InputShim::using(source);
        let res = shim.read();
        assert_eq!(res, Ok('A' as u8));
    }

    #[test]
    fn reads_first() {
        let source: Box<dyn Read> = Box::new("Hello, world!".as_bytes());
        let mut shim = InputShim::using(source);
        let res = shim.read();
        assert_eq!(res, Ok('H' as u8));
    }

    #[test]
    fn reads_multiple() {
        let source: Box<dyn Read> = Box::new("Hello, world!".as_bytes());
        let mut shim = InputShim::using(source);
        let mut res = shim.read();
        assert_eq!(res, Ok('H' as u8));
        res = shim.read();
        assert_eq!(res, Ok('e' as u8));
        res = shim.read();
        assert_eq!(res, Ok('l' as u8));
        res = shim.read();
        assert_eq!(res, Ok('l' as u8));
    }

    #[test]
    fn read_too_many() {
        let source: Box<dyn Read> = Box::new("Hi".as_bytes());
        let mut shim = InputShim::using(source);
        shim.read();
        shim.read();
        let res = shim.read();
        assert_eq!(res, Err(ReadError));
    }
}
