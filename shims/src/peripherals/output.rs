use lc3_traits::peripherals::output::{Output, WriteError};
use std::io::{Write, stdout};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Shim {
    sink: Rc<RefCell<dyn Write>>
}

impl Default for Shim {
    fn default() -> Self {
        Self {
            sink: Rc::new(RefCell::new(stdout()))
        }
    }
}

impl Shim {
    fn new() -> Self {
        Self::default()
    }
    
    fn using(sink: Rc<RefCell<dyn Write>>) -> Self {
        Self { sink }
    }
}

impl Output for Shim {
    fn write(&mut self, c: u8) -> Result<(), WriteError> {
        let ret = match self.sink.borrow_mut().write(&[c]) {
            Ok(1) => Ok(()), 
            _ => Err(WriteError),
        };
        match self.sink.borrow_mut().flush() {
            Ok(()) => ret,
            _ => Err(WriteError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn write_one() {
        let sink = Rc::new(RefCell::new(Vec::new()));
        let mut shim = Shim::using(sink.clone());
        let ch0 = 'A' as u8;
        let res = shim.write(ch0);
        assert_eq!(res, Ok(()));
        assert_eq!(sink.borrow()[0], ch0);
    }

    #[test]
    fn write_multiple() {
        let sink = Rc::new(RefCell::new(Vec::new()));
        let mut shim = Shim::using(sink.clone());
        let ch0 = 'L' as u8;
        let ch1 = 'C' as u8;
        let ch2 = '-' as u8;
        let ch3 = '3' as u8;
        shim.write(ch0);
        shim.write(ch1);
        shim.write(ch2);
        shim.write(ch3);
        assert_eq!(sink.borrow()[0], ch0);
        assert_eq!(sink.borrow()[1], ch1);
        assert_eq!(sink.borrow()[2], ch2);
        assert_eq!(sink.borrow()[3], ch3);
    }
    
    // Making a fixed-length Write is harder than it seems.
//    #[test]
//    fn write_too_much() {
//        let mut buf: [u8; 1] = [0];
//        let sink = Rc::new(RefCell::new(buf));
//        let mut shim = Shim::using(sink.clone());
//        let ch0 = 'Y' as u8;
//        let ch1 = 'P' as u8;
//        shim.write(ch0);
//        let res = shim.write(ch1);
//        assert_eq!(res, Err(WriteError));
//    }
}

