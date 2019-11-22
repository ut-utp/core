use lc3_traits::peripherals::input::{Input, ReadError};
use std::io::{stdin, Read};

pub struct InputShim {
    source: Box<dyn Read>,
}

impl Default for InputShim {
    fn default() -> Self {
        Self {
            source: Box::new(stdin()),
        }
    }
}

impl InputShim {
    fn new() -> Self {
        Self::default()
    }

    fn using(source: Box<dyn Read>) -> Self {
        Self { source }
    }
}

impl Input for InputShim {
    fn read(&mut self) -> Result<u8, ReadError> {
        let mut buf: [u8; 1] = [0];
        match self.source.read(&mut buf) {
            Ok(0) => Err(ReadError),
            Ok(_) => Ok(buf[0]),
            Err(_) => Err(ReadError),
        }
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
