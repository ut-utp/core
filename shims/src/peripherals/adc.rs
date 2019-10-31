use lc3_traits::peripherals::adc::{self, Adc, Pin, PinArr, ReadError, NUM_PINS, StateMismatch, Handler};

// TODO: consider using one array of struct(Option<u8>, State, &'a dyn FnMut(u8))?
pub struct Shim<'a> {
    states:   PinArr<State>,
    handlers: PinArr<&'a Handler>,
}

#[derive(Copy, Clone)]
pub enum State {
    Enabled  (u8),
    Interrupt(u8),
    Disabled,
}

impl From<State> for adc::State {
    fn from(state: State) -> adc::State {
        use adc::State::*;
        match state {
            State::Enabled  (_) => Enabled,
            State::Interrupt(_) => Interrupt,
            State::Disabled     => Disabled,
        }
    }
}

const INIT_VALUE: u8 = 0;
const NO_OP: &Handler = &|_| {};

impl Default for Shim<'_> {
    fn default() -> Self {
        Self {
            states:   PinArr([State::Disabled; NUM_PINS as usize]),
            handlers: PinArr([NO_OP;           NUM_PINS as usize]),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SetError(StateMismatch);

impl Shim<'_> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_value(&mut self, pin: Pin, value: u8) -> Result<(), SetError> {
        use State::*;
        
        self.states[pin] = match self.states[pin] {
            Enabled(_) => Enabled(value),
            Interrupt(prev) => {
                if prev != value {
                    self.handlers[pin](value)
                }
                Interrupt(value)
            }
            _ => return Err(SetError((pin, self.get_state(pin))))
        };
        Ok(())        
    }
}

impl<'a> Adc<'a> for Shim<'a> {
    fn set_state(&mut self, pin: Pin, state: adc::State) -> Result<(), ()> {
        use adc::State::*;
        self.states[pin] = match state {
            Enabled   => State::Enabled  (INIT_VALUE),
            Interrupt => State::Interrupt(INIT_VALUE),
            Disabled  => State::Disabled,
        };
        Ok(())
    }

    fn get_state(&self, pin: Pin) -> adc::State {
        self.states[pin].into()
    }

    fn read(&self, pin: Pin) -> Result<u8, ReadError> {
        use State::*;
        match self.states[pin] {
            Enabled(value) | Interrupt(value) => Ok(value),
            valueless => Err(ReadError((pin, valueless.into()))),
        }
    }

    fn register_interrupt(
        &mut self,
        pin: Pin,
        handler: &'a Handler
    ) -> Result<(), ()> {
        self.handlers[pin] = handler;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::adc::{self, Adc, Pin::*,};

    #[test]
    fn get_state_initial() {
        let shim = Shim::new();
        assert_eq!(shim.get_state(A0), adc::State::Disabled)
    }
    
    #[test]
    fn read_initial() {
        let mut shim = Shim::new();
        let res = shim.set_state(A0, adc::State::Enabled);
        assert_eq!(res, Ok(()));
        let val = shim.read(A0);
        assert_eq!(val, Ok(INIT_VALUE));
    }
    
    #[test]
    fn set_value() {
        let new_val: u8 = 1;
        assert_ne!(INIT_VALUE, new_val, "TEST FAULTY: new_val must not equal INIT_VALUE");
        let mut shim = Shim::new();
        shim.set_state(A0, adc::State::Enabled);
        let res = shim.set_value(A0, new_val);
        assert_eq!(res, Ok(()));
        let val = shim.read(A0);
        assert_eq!(val, Ok(new_val));
    }
    
    #[test]
    fn read_disabled() {
        let mut shim = Shim::new();
        let val = shim.read(A0);
        assert_eq!(val, Err(ReadError((A0, adc::State::Disabled))))
    }
}

