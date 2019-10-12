
use std::sync::{Arc, RwLock};
use core::ops::{Index, IndexMut};
use crate::peripherals::gpio::{Gpio, GpioPin, GpioState, GpioPinArr, GpioReadError, GpioWriteError, GpioMiscError, NUM_GPIO_PINS};

#[derive(Copy, Clone, Debug)]
pub enum State {
    Input(bool),
    Output(bool),
    Interrupt(bool),
    Disabled,
}

impl From<State> for GpioState {
    fn from(state: State) -> GpioState {
        use GpioState::*;

        match state {
            State::Input(_) => Input,
            State::Output(_) => Output,
            State::Interrupt(_) => Interrupt,
            State::Disabled => Disabled,
        }
    }
}

impl From<GpioPin> for usize {
    fn from(pin: GpioPin) -> usize {
        use GpioPin::*;

        match pin {
            G0 => 0,
            G1 => 1,
            G2 => 2,
            G3 => 3,
            G4 => 4,
            G5 => 5,
            G6 => 6,
            G7 => 7,
        }
    }
}

/// A simple reference implementation of the [`Gpio` peripheral
/// trait](crate::peripherals::Gpio).
///
/// Some implementation details:
///   - The value of a pin is set to 0 when the pin is switched to input,
///     output, or interrupt mode.
///   - The value of a pin can be read when the pin is in input, output, or
///     interrupt mode (anything but disabled).
///   - The value of a pin can be _set_ when the pin is in input or interrupt
///     mode.
///   - The state of a pin (input, output, interrupt, or disabled) can be
///     retrieved at any time.
pub struct GpioShim<'a> {
    states: GpioPinArr<State>,
    handlers: GpioPinArr<&'a dyn Fn(GpioPin)>
    // handlers: GpioPinArr<Box<dyn Fn(GpioPin)>>
    // handlers: GpioPinArr<Box<dyn FnMut(GpioPin)>>
    // handlers: GpioPinArr<&'static dyn FnMut(GpioPin)>
    // handlers: GpioPinArr<&'a dyn FnMut(GpioPin)>,
}

impl Index<GpioPin> for GpioShim<'_> {
    type Output = State;

    fn index(&self, pin: GpioPin) -> &State {
        &self.states[Into::<usize>::into(pin)]
    }
}

impl IndexMut<GpioPin> for GpioShim<'_> {
    fn index_mut(&mut self, pin: GpioPin) -> &mut State {
        &mut self.states[Into::<usize>::into(pin)]
    }
}

// static no_op: &dyn Fn(GpioPin) = &|_| {};
const NO_OP: &dyn Fn(GpioPin) = &|_| {};

impl Default for GpioShim<'_> {
    fn default() -> Self {
        Self {
            states: [State::Disabled; NUM_GPIO_PINS as usize],
            // handlers: [Box::new(&|_| {}); NUM_GPIO_PINS as usize],
            // handlers: [no_op; NUM_GPIO_PINS as usize],
            handlers: [NO_OP; NUM_GPIO_PINS as usize],
        }
    }
}

impl GpioShim<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_shared() -> Arc<RwLock<Self>> {
        Arc::<RwLock<Self>>::default()
    }

    /// Sets a pin if it's in input or interrupt mode.
    ///
    /// Returns `Some(())` on success and `None` on failure.
    pub fn set_pin(&mut self, pin: GpioPin, bit: bool) -> Option<()> {
        use State::*;

        self[pin] = match self[pin] {
            Input(_) => Input(bit),
            Interrupt(prev) => {
                // Rising edge!
                if bit && !prev {
                    self.handlers[Into::<usize>::into(pin)](pin)
                }

                Interrupt(bit)
            },
            Output(_) | Disabled => return None,
        };

        Some(())
    }

    /// Gets the value of a pin.
    ///
    /// Returns `None` when the pin is disabled.
    pub fn get_pin(&self, pin: GpioPin) -> Option<bool> {
        use State::*;

        match self[pin] {
            Input(b) | Output(b) | Interrupt(b) => Some(b),
            Disabled => None,
        }
    }

    pub fn get_pin_state(&self, pin: GpioPin) -> GpioState {
        self[pin].into()
    }
}

// impl Arc<RwLock<GpioShim>> {
//     fn new() -> Self {
//         Self::default()
//     }
// }

impl<'a> Gpio<'a> for GpioShim<'a> {
    fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError> {
        use GpioState::*;

        self[pin] = match state {
            Input => State::Input(false),
            Output => State::Output(false),
            Interrupt => State::Interrupt(false),
            Disabled => State::Disabled,
        };

        Ok(())
    }

    fn get_state(&self, pin: GpioPin) -> GpioState {
        self.get_pin_state(pin)
    }

    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> {
        use State::*;

        if let Input(b) | Interrupt(b) = self[pin] {
            Ok(b)
        } else {
            Err(GpioReadError((pin, self[pin].into())))
        }
    }

    fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError> {
        use State::*;

        if let Output(_) = self[pin] {
            self[pin] = Output(bit);
            Ok(())
        } else {
            Err(GpioWriteError((pin, self[pin].into())))
        }
    }

    // fn register_interrupt<'a>(&'a mut self, pin: GpioPin, handler: impl FnMut(GpioPin) + 'a) -> Result<(), GpioMiscError> {
    //     // self.handlers[Into::<usize>::into(pin)] = Box::new(handler);
    //     self.handlers[Into::<usize>::into(pin)] = Box::new(handler);
    //     // self.handlers[Into::<usize>::into(pin)] = &handler;

    //     Ok(())
    // }

    fn register_interrupt(&mut self, pin: GpioPin, handler: &'a dyn Fn(GpioPin)) -> Result<(), GpioMiscError> {
        self.handlers[Into::<usize>::into(pin)] = handler;

        Ok(())
    }
}

