//! [`Gpio` trait](Gpio) and friends.

use core::convert::TryFrom;

use crate::peripheral_trait;

// Switched to using enums to identify peripheral pin numbers; this way
// referring to invalid/non-existent pin numbers isn't an error that peripheral
// trait implementations have to deal with.
//
// This seems to make more since, if you consider that the peripherals are
// exposed through a memory-mapped interface an invalid pin number isn't really
// an error that can happen (you either hit a memory address that corresponds
// to a peripheral or you hit an invalid memory address).
//
// This is currently a little wonky, but it'll be better once we write the macro
// described in `control.rs`.

#[rustfmt::skip]
#[derive(Copy, Clone)]
pub enum GpioPin { G0, G1, G2, G3, G4, G5, G6, G7 }
const NUM_GPIO_PINS: u8 = 8; // G0 - G7; TODO: derive macro (also get it to impl Display)
const GPIO_PINS: [GpioPin; NUM_GPIO_PINS as usize] = {
    use GpioPin::*;
    [G0, G1, G2, G3, G4, G5, G6, G7]
}; // TODO: once we get the derive macro, get rid of this.

#[derive(Copy, Clone)]
pub enum GpioState {
    Input,
    Output,
    Interrupt, // TBD: Can you call read on a pin configured for interrupts?
    Disabled,
}

pub struct GpioMiscError;

type GpioStateMismatch = (GpioPin, GpioState);

#[derive(Copy, Clone)]
pub struct GpioReadError(GpioStateMismatch);
#[derive(Copy, Clone)]
pub struct GpioWriteError(GpioStateMismatch);

type GpioPinArr<T> = [T; NUM_GPIO_PINS as usize];

type GpioStateMismatches = GpioPinArr<Option<GpioStateMismatch>>; // [Option<GpioStateMismatch>; NUM_GPIO_PINS as usize];

#[derive(Copy, Clone)]
pub struct GpioReadErrors(GpioStateMismatches);
#[derive(Copy, Clone)]
pub struct GpioWriteErrors(GpioStateMismatches);

// #[derive(Copy, Clone)]
// pub struct GpioInterruptRegisterError(GpioStateMismatch); // See comments below

// trace_macros!(true);

peripheral_trait! {gpio,
pub trait Gpio {
    fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError>; // should probably be infallible
    fn get_state(&self, pin: GpioPin) -> GpioState;
    fn get_states(&self) -> GpioPinArr<GpioState> {
        let mut states = [GpioState::Disabled; NUM_GPIO_PINS as usize]; // TODO (again)

        GPIO_PINS
            .iter()
            .enumerate()
            .for_each(|(idx, g)| states[idx] = self.get_state(*g));

        states
    }

    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError>; // errors on state mismatch (i.e. you tried to read but the pin is configured as an output)
    fn read_all(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        // TODO: here's a thing; [Result<bool, GpioReadError>] or Result<[bool], [GpioReadError]>?
        // The interpreter will _probably_ just use a default value upon encountering read errors
        // meaning that we don't want to do the latter which is all or nothing (i.e. if some of the
        // reads worked, give us their values! We'll use them!).

        let mut readings = [Ok(false); NUM_GPIO_PINS as usize]; // TODO: it's weird and gross that we have to use a default value here (derive macro save us pls!!)

        GPIO_PINS
            .iter()
            .enumerate()
            .for_each(|(idx, g)| readings[idx] = self.read(*g));

        readings
    }

    fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError>; // errors on state mismatch
    fn write_all(&mut self, bits: GpioPinArr<bool>) -> GpioPinArr<Result<(), GpioWriteError>> {
        // TODO: return an array of results or one result?
        // For the actual interpreter, it doesn't make a difference; we have no mechanism by which
        // we even communicate errors to the LC-3 program. But the debugger can communicate this kind
        // of stuff so let's not throw the information away.

        let mut errors = [Ok(()); NUM_GPIO_PINS as usize];

        GPIO_PINS
            .iter()
            .zip(bits.iter())
            .enumerate()
            .for_each(|(idx, (pin, bit))| {
                errors[idx] = self.write(*pin, *bit);
            });

        errors
    }

    // This error only make sense if you have to put the Gpio Pin in interrupt mode _before_ you set the interrupt handler.
    // That doesn't really make any sense.
    //
    // This operation should probably be infallible. If we want to actually check that a handler has been registered, we could require that
    // the handler be registered first and then when you call set_state, it can error if it's still using the default handler.
    //
    // But really, enabling interrupts and having them go to the default handler should be possible... (default handler should probably do nothing!)
    //
    // Another approach is to make adding interupts an extra thing that you can do when you're in Input mode. I don't like this because
    // it means we now need to provide a disable_interrupt function though...
    // fn register_interrupt(&mut self, pin: GpioPin, func: impl FnMut(bool)) -> Result<(), GpioInterruptRegisterError>;

    // Gonna switch to MiscError for now then (TODO ^^^^^^):
    fn register_interrupt(
        &mut self,
        pin: GpioPin,
        func: impl FnMut(bool)
    ) -> Result<(), GpioMiscError>;
}}

impl TryFrom<GpioPinArr<Result<bool, GpioReadError>>> for GpioReadErrors {
    type Error = ();

    fn try_from(
        read_errors: GpioPinArr<Result<bool, GpioReadError>>,
    ) -> Result<GpioReadErrors, ()> {
        if read_errors.iter().all(|r| r.is_ok()) {
            Err(()) // No error!
        } else {
            let mut errors: GpioStateMismatches = [None; NUM_GPIO_PINS as usize];

            read_errors
                .iter()
                .enumerate()
                .filter_map(|(idx, res)| {
                    res.map_err(|gpio_read_error| (idx, gpio_read_error)).err()
                })
                .for_each(|(idx, gpio_read_error)| {
                    errors[idx] = Some(gpio_read_error.0);
                });

            Ok(GpioReadErrors(errors))
        }
    }
}

impl TryFrom<GpioPinArr<Result<(), GpioWriteError>>> for GpioWriteErrors {
    type Error = ();

    fn try_from(
        write_errors: GpioPinArr<Result<(), GpioWriteError>>,
    ) -> Result<GpioWriteErrors, ()> {
        if write_errors.iter().all(|w| w.is_ok()) {
            // None
            Err(())
        } else {
            let mut errors: GpioStateMismatches = [None; NUM_GPIO_PINS as usize];

            write_errors
                .iter()
                .enumerate()
                .filter_map(|(idx, res)| {
                    res.map_err(|gpio_write_error| (idx, gpio_write_error))
                        .err()
                })
                .for_each(|(idx, gpio_write_error)| {
                    errors[idx] = Some(gpio_write_error.0);
                });

            // Some(GpioWriteErrors(errors))
            Ok(GpioWriteErrors(errors))
        }
    }
}

// use crate::peripheral_set_impl;

// Impl for PeripheralSet
// peripheral_set_impl!(Gpio, {
//     fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError> {
//         self.gpio.set_state(pin, state)
//     }
//     fn get_state(&self, pin: GpioPin) -> GpioState { self.gpio.get_state(pin) }
//     fn get_states(&self) -> GpioPinArr<GpioState> { self.gpio.get_states() }

//     fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> { self.gpio.read(pin) }
//     fn read_all(&self) -> GpioPinArr<Result<bool, GpioReadError>> { self.gpio.read_all() }

//     fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError> {
//         self.gpio.write(pin, bit)
//     }
//     fn write_all(&mut self, bits: GpioPinArr<bool>) -> GpioPinArr<Result<(), GpioWriteError>> {
//        self.gpio.write_all(bits)
//     }

//     fn register_interrupt(&mut self, pin: GpioPin, func: impl FnMut(bool)) -> Result<(), GpioMiscError> {
//         self.gpio.register_interrupt(pin, func)
//     }
// });
