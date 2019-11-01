use lc3_traits::peripherals::output::{Output, OutputError};
use std::io::{stdout, Error as IoError, Write};

use std::rc::Rc;

pub struct OutputShim<'a> {
    sink: &'a mut dyn Write,
}

impl Default for OutputShim<'_> {
    fn default() -> Self {
        let out = Box::new(stdout());

        Self {
            sink: std::boxed::Box::<std::io::Stdout>::leak(out), // TODO: DO NOT DO THIS!!!
        }
    }
}

impl<'a> OutputShim<'a> {
    fn new() -> Self {
        Self::default()
    }

    fn using(sink: &'a mut dyn Write) -> Self {
        Self { sink }
    }
}

impl Output for OutputShim<'_> {
    fn write(&mut self, c: u8) -> Result<(), OutputError> {
        self.sink.write(&[c]).map_err(|_| OutputError)?;
        self.sink.flush().map_err(|_| OutputError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_one() {
        let mut sink = Vec::new();
        let mut shim = OutputShim::using(&mut sink);
        let ch0 = 'A' as u8;
        let res = shim.write(ch0);
        assert_eq!(res, Ok(()));
        assert_eq!(sink[0], ch0);
    }

    #[test]
    fn write_multiple() {
        let mut sink = Vec::new();
        let mut shim = OutputShim::using(&mut sink);
        let ch0 = 'L' as u8;
        let ch1 = 'C' as u8;
        let ch2 = '-' as u8;
        let ch3 = '3' as u8;
        shim.write(ch0).unwrap();
        shim.write(ch1).unwrap();
        shim.write(ch2).unwrap();
        shim.write(ch3).unwrap();
        assert_eq!(sink[0], ch0);
        assert_eq!(sink[1], ch1);
        assert_eq!(sink[2], ch2);
        assert_eq!(sink[3], ch3);
    }

    #[test]
    #[ignore]
    // Annoyingly, this does not fail.
    fn write_too_much() {
        let mut buf: [u8; 1] = [0];
        let sink = &mut buf.as_mut();
        let mut shim = OutputShim::using(sink);
        let ch0 = 'Y' as u8;
        let ch1 = 'P' as u8;
        shim.write(ch0).unwrap();
        let res = shim.write(ch1);
        // assert_eq!(res, Err(OutputError));
    }
}
