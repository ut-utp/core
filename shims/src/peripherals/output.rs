use lc3_traits::peripherals::output::{Output, OutputError};
use std::io::{stdout, Error as IoError, Write};

use std::convert::AsMut;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

enum OwnedOrRef<'a, R: ?Sized> {
    Owned(Box<R>),
    Ref(&'a mut R),
}

impl<'a, R: ?Sized> Deref for OwnedOrRef<'a, R> {
    type Target = R;

    fn deref(&self) -> &R {
        use OwnedOrRef::*;

        match self {
            Owned(r) => r,
            Ref(r) => r,
        }
    }
}

impl<'a, R: ?Sized> DerefMut for OwnedOrRef<'a, R> {
    fn deref_mut(&mut self) -> &mut R {
        use OwnedOrRef::*;

        match self {
            Owned(r) => r,
            Ref(r) => r,
        }
    }
}

pub struct OutputShim<'a, 'b> {
    // pub struct OutputShim {
    // sink: &'a mut dyn Write,
    // sink: Box<dyn Write>,
    sink: OwnedOrRef<'a, dyn Write + 'a>,
    // _marker: PhantomData<&'a ()>,
    // sink: &'a mut dyn AsRef<dyn Write>,
    flag: Option<&'b AtomicBool>,
    interrupts_enabled: bool,
}

impl Default for OutputShim<'_, '_> {
    // impl Default for OutputShim {
    fn default() -> Self {
        Self {
            // sink: std::boxed::Box::<std::io::Stdout>::leak(out), // TODO: DO NOT DO THIS!!!
            sink: OwnedOrRef::Owned(Box::new(stdout())),
            flag: None,
            interrupts_enabled: false,
        }
    }
}

impl<'a, 'b> OutputShim<'a, 'b> {
    // impl OutputShim {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn using(sink: Box<dyn Write>) -> Self {
        Self {
            sink: OwnedOrRef::Owned(sink),
            flag: None,
            interrupts_enabled: false,
        }
    }

    pub fn with_ref(sink: &'a mut dyn Write) -> Self {
        Self {
            sink: OwnedOrRef::<'a, dyn Write>::Ref(sink),
            flag: None,
            interrupts_enabled: false,
        }
    }
}

impl<'b> Output<'b> for OutputShim<'_, 'b> {
    // impl Output for OutputShim {
    fn write(&mut self, c: u8) -> Result<(), OutputError> {
        let _ = self.sink.write(&[c]).map_err(|_| OutputError)?;
        self.sink.flush().map_err(|_| OutputError)?;
        if self.interrupts_enabled() {
            match self.flag {
                Some(f) => f.store(true, Ordering::SeqCst),
                None => unreachable!(),
            }
        }
        Ok(())
    }

    fn register_interrupt_flag(&mut self, flag: &'b AtomicBool) {
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
