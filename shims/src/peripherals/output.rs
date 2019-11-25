use lc3_traits::peripherals::output::{Output, OutputError};
use std::io::{stdout, Error as IoError, Write};

use std::convert::AsMut;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::peripherals::OwnedOrRef;

pub struct OutputShim<'a, 'b> {
    sink: OwnedOrRef<'a, dyn Write + 'a>,
    flag: Option<&'b AtomicBool>,
    interrupt_enable_bit: bool,
}

impl Default for OutputShim<'_, '_> {
    fn default() -> Self {
        Self::using(Box::new(stdout()))
    }
}

impl<'a, 'b> OutputShim<'a, 'b> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn using(sink: Box<dyn Write>) -> Self {
        Self {
            sink: OwnedOrRef::Owned(sink),
            flag: None,
            interrupt_enable_bit: false,
        }
    }

    pub fn with_ref(sink: &'a mut dyn Write) -> Self {
        Self {
            sink: OwnedOrRef::<'a, dyn Write>::Ref(sink),
            flag: None,
            interrupt_enable_bit: false,
        }
    }
}

impl<'b> Output<'b> for OutputShim<'_, 'b> {
    fn register_interrupt_flag(&mut self, flag: &'b AtomicBool) {
        self.flag = match self.flag {
            None => Some(flag),
            Some(_) => unreachable!(),
        }
    }

    fn interrupt_occurred(&self) -> bool {
        self.current_data_written()
    }
    
    fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupt_enable_bit = bit;
    }
    
    fn interrupts_enabled(&self) -> bool {
        self.interrupt_enable_bit
    }
    
    // TODO: handle OutputErrors to somehow report that the write or flush went wrong
    fn write_data(&mut self, c: u8) -> Result<(), OutputError> {
        match self.flag {
            Some(f) => f.store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
        self.sink.write(&[c]).map_err(|_| OutputError)?;
        self.sink.flush().map_err(|_| OutputError)?;
        match self.flag {
            Some(f) => f.store(true, Ordering::SeqCst),
            None => unreachable!(),
        }
        Ok(())
    }

    fn current_data_written(&self) -> bool {
        match self.flag {
            Some(f) => f.load(Ordering::SeqCst),
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_one() {
        let mut sink = Vec::new();
        let mut shim = OutputShim::with_ref(&mut sink);
        // let mut shim = OutputShim { sink: OwnedOrRef::Ref(&mut sink) };
        let ch0 = 'A' as u8;
        let res = shim.write(ch0);
        drop(shim);
        assert_eq!(res, Ok(()));
        assert_eq!(sink[0], ch0);
    }

    #[test]
    fn write_multiple() {
        let mut sink = Vec::new();
        let mut shim = OutputShim::with_ref(&mut sink);
        let ch0 = 'L' as u8;
        let ch1 = 'C' as u8;
        let ch2 = '-' as u8;
        let ch3 = '3' as u8;
        shim.write(ch0).unwrap();
        shim.write(ch1).unwrap();
        shim.write(ch2).unwrap();
        shim.write(ch3).unwrap();
        drop(shim);
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
        let mut shim = OutputShim::with_ref(sink);
        let ch0 = 'Y' as u8;
        let ch1 = 'P' as u8;
        shim.write(ch0).unwrap();
        let res = shim.write(ch1);
        assert_eq!(res, Err(OutputError));
    }
}
