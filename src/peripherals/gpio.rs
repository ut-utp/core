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
pub(crate) const NUM_GPIO_PINS: u8 = 8; // G0 - G7; TODO: derive macro (also get it to impl Display)
pub(crate) const GPIO_PINS: [GpioPin; NUM_GPIO_PINS as usize] = {
    use GpioPin::*;
    [G0, G1, G2, G3, G4, G5, G6, G7]
}; // TODO: once we get the derive macro, get rid of this.

#[derive(Copy, Clone)]
pub enum GpioState {
    Input,
    Output,
    Interrupt, // TBD: Can you call read on a pin configured for interrupts? (TODO)
               // Tentatively, yes.
               //
               // 00 -> Disabled
               // 01 -> Output
               // 10 -> Input
               // 11 -> Interrupt (Rising Edge)
    Disabled,
}

pub struct GpioMiscError;

type GpioStateMismatch = (GpioPin, GpioState);

#[derive(Copy, Clone)]
pub struct GpioReadError(GpioStateMismatch);
#[derive(Copy, Clone)]
pub struct GpioWriteError(GpioStateMismatch);

pub(crate) type GpioPinArr<T> = [T; NUM_GPIO_PINS as usize];

pub(crate) type GpioStateMismatches = GpioPinArr<Option<GpioStateMismatch>>; // [Option<GpioStateMismatch>; NUM_GPIO_PINS as usize];

#[derive(Copy, Clone)]
pub struct GpioReadErrors(GpioStateMismatches);
#[derive(Copy, Clone)]
pub struct GpioWriteErrors(GpioStateMismatches);

// #[derive(Copy, Clone)]
// pub struct GpioInterruptRegisterError(GpioStateMismatch); // See comments below

peripheral_trait! {gpio,
/// GPIO access trait.
///
/// Implementations of this trait must provide digital read, digital write, and rising
/// edge trigger interrupt functionality for 8 GPIO pins which we'll call G0 - G7.
///
/// Additionally, implementors of this trait must also provide an implementation of
/// [`Default`](core::default::Default).
///
/// ### State
/// The interpreter (user of this trait) will set the states of all the pins to
/// [`GpioState::Disabled`] on startup, so implementations can choose any default state
/// they wish.
///
/// Implementations should maintain the state of the GPIO pins and querying this state
/// ([`get_state`]) should be an infallible operation.
///
/// Setting pin state ([`set_state`]) is not infallible as implementations may change
/// need to actually change the state of hardware peripherals in order to, for example,
/// register a rising-edge interrupt for a particular pin. Though implementors are
/// encouraged to make this operation infallible if possible, we realize this isn't
/// always possible and in the event that it isn't, we'd rather have implementors pass
/// the error onto the interpreter instead of panicking.
///
/// ### Reads and Writes
/// Reading from pins should fail (with a [`GpioReadError`]) when pins are disabled or
/// in output ([`GpioState::Output`]) mode. *Note:* reading from pins in interrupt
/// ([`GpioState::Interrupt`]) mode is allowed.
///
/// Writing from pins should fail (with a [`GpioWriteError`]) when pins are disabled or
/// in input ([`GpioState::Input`]) or interrupt ([`GpioState::Interrupt`]) mode.
///
/// ### Interrupts
/// Registering interrupts (i.e. calling [`register_interrupt`]) does not automatically
/// put a pin in [`interrupt`](GpioState::Interrupt) mode. Instead, this only updates
/// the handler function for a pin.
///
/// Handler functions are `FnMut` implementors (they're allowed to mutate state) that
/// take a [`GpioPin`] corresponding to the pin for which the rising-edge interrupt just
/// fired.
///
/// Implementations should store the last handler function provided to
/// [`register_interrupt`] _across pin state changes_. As in, if G0 (GPIO pin 0)'s
/// handler is set to function A (i.e. `register_interrupt(GpioPin::G0, A)`), and then
/// G0's state is changed to [`output`](GpioState::Output) and then to
/// [`disabled`](GpioState::Disabled) and then to [`interrupt`](GpioState::Interrupt), A
/// should be called when G0 goes from low to high.
///
/// Implementors should use a no-op handler (do nothing) for the pins by default. All
/// users of this trait _should_ register handlers on initialization (just as they will
/// explicitly set the state of all pins to [disabled](GpioState::Disabled)), but
/// implementors shouldn't bank on this.
///
/// As is probably obvious, implementors should only call the handler function on a
/// rising edge *if the pin is in [interrupt](GpioState::Interrupt) mode* (not just if
/// a handler function has been provided).
///
/// ### Default Function Implementations
/// The trait provides naïve default implementations of [`get_states`], [`read_all`],
/// and [`write_all`] that just call their single pin variants across all pins; as an
/// implementor you can choose to override these if you wish. If there's an easier way
/// to do a particular operation across all the pins than just calling the single pin
/// variant in a loop, then it's probably worth doing; i.e. if you happen to store
/// [`GpioState`]s for the pins in an array, you could override [`get_states`] to just
/// return your array pretty easily. Otherwise, the default implementations should work
/// fine.
///
/// ### Tests
/// There are [tests for this trait](crate::tests::gpio) in the [tests
/// module](crate::tests) to help ensure that your implementation of this trait follows
/// the rules above.
pub trait Gpio: Default {

    /// Yo
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
    // Another approach is to make adding interrupts an extra thing that you can do when you're in Input mode. I don't like this because
    // it means we now need to provide a disable_interrupt function though...
    // fn register_interrupt(&mut self, pin: GpioPin, func: impl FnMut(bool)) -> Result<(), GpioInterruptRegisterError>;

    // Gonna switch to MiscError for now then (TODO ^^^^^^):
    fn register_interrupt(
        &mut self,
        pin: GpioPin,
        handler: impl FnMut(GpioPin)
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
