use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};

use lc3_traits::peripherals::{Gpio, Adc, Pwm, Timers, Clock, Input, Output};

use lc3_traits::peripherals::gpio::{GpioPin, GpioState, GpioPinArr, GpioReadError, GpioWriteError, GpioMiscError};

// Gonna do this for pins for now:
//
//           [For reference]            |              [Assignments]           |
//                                      |                                      |
//     J1    J3           J2    J4      |     J1    J3           J2    J4      |
//   ┏━━━━━┯━━━━━┓      ┏━━━━━┯━━━━━┓   |   ┏━━━━━┯━━━━━┓      ┏━━━━━┯━━━━━┓   |
//   ┃ 3.3 │ 5.0 ┃      ┃ PF2 │ GND ┃   |   ┃ 3.3 │ 5.0 ┃      ┃  P0 │ GND ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PB5 │ GND ┃      ┃ PF3 │ PB2 ┃   |   ┃  G5 │ GND ┃      ┃  P1 │  G2 ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PB0 │ PD0 ┃      ┃ PB3 │ PE0 ┃   |   ┃  G0 │ XXX ┃      ┃  G3 │  A0 ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PB1 │ PD1 ┃      ┃ PC4 │ PF0 ┃   |   ┃  G1 │ XXX ┃      ┃ PC4 │  Px ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PE4 │ PD2 ┃      ┃ PC5 │ RST ┃   |   ┃  A4 │ PD2 ┃      ┃ PC5 │ RST ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PE5 │ PD3 ┃      ┃ PC6 │ PB7 ┃   |   ┃  A5 │ PD3 ┃      ┃ PC6 │  G7 ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PB4 │ PE1 ┃      ┃ PC7 │ PB6 ┃   |   ┃  G4 │  A1 ┃      ┃ PC7 │  G6 ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PA5 │ PE2 ┃      ┃ PD6 │ PA4 ┃   |   ┃ PA5 │  A2 ┃      ┃ PD6 │ PA4 ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PA6 │ PE3 ┃      ┃ PD7 │ PA3 ┃   |   ┃ PA6 │  A3 ┃      ┃ PD7 │ PA3 ┃   |
//   ┠─────┼─────┨      ┠─────┼─────┨   |   ┠─────┼─────┨      ┠─────┼─────┨   |
//   ┃ PA7 │ PF1 ┃      ┃ PF4 │ PA2 ┃   |   ┃ PA7 │  Px ┃      ┃ PF4 │ PA2 ┃   |
//   ┗━━━━━┷━━━━━┛      ┗━━━━━┷━━━━━┛   |   ┗━━━━━┷━━━━━┛      ┗━━━━━┷━━━━━┛   |
//         ┏━━━━━┓      ┏━━━━━┓         |         ┏━━━━━┓      ┏━━━━━┓         |
//         ┃ GND ┃      ┃ GND ┃         |         ┃ GND ┃      ┃ GND ┃         |
//         ┠─────┨      ┠─────┨         |         ┠─────┨      ┠─────┨         |
//         ┃ GND ┃      ┃ GND ┃         |         ┃ GND ┃      ┃ GND ┃         |
//         ┠─────┨      ┠─────┨         |         ┠─────┨      ┠─────┨         |
//         ┃ 5.0 ┃      ┃ 3.3 ┃         |         ┃ 5.0 ┃      ┃ 3.3 ┃         |
//         ┗━━━━━┛      ┗━━━━━┛         |         ┗━━━━━┛      ┗━━━━━┛         |
//   ┏━━━━━┯━━━━━┓      ┏━━━━━┓         |   ┏━━━━━┯━━━━━┓      ┏━━━━━┓         |
//   ┃ PD0<->PB6 ┃      ┃D7^VD┃         |   ┃ PD0<->PB6 ┃      ┃D7^VD┃         |
//   ┠─────┼─────┨      ┗━━━━━┛         |   ┠─────┼─────┨      ┗━━━━━┛         |
//   ┃ PD1<->PB7 ┃                      |   ┃ PD1<->PB7 ┃                      |
//   ┗━━━━━┷━━━━━┛                      |   ┗━━━━━┷━━━━━┛                      |
//                                      |                                      |
//--------------------------------------|--------------------------------------|
//
// For GPIO:
//   - We want a full port so that `write_all` is easier.
//      + Port A is out because A0 and A1 are UART.
//      + Port C is out because C0 - C3 are used for debugging.
//      + Port E and F are out because they don't have 8 exposed pins.
//      + Port D's PD4 and PD5 are connected to USB and not connected to pins.
//      + This leaves Port B. We'll be sure to disable PD0 and PD1 so board with
//        the shunt resistors still work.
//  - Another way to go about this would be to assign pin mappings based on the
//    physical location of pins (i.e. take all of J1 to be GPIO).
//      + For now, we won't do this.
//  - Another consideration is that it would be nice for the two buttons (PF0
//    and PF4) to be mapped to GPIO pins so users can use the buttons as an
//    input source.
//      + Whether this is more desirable than having a unified port is a
//        question for another time.
//
// For ADC:
//  - We need 6 pins.
//  - Our options are all of Port E (6 pins), 2 pins in Port B, and PD0 to PD3.
//  - Again, we'll opt for matching the numbering (Port E).
// // - Our options are Port B, Port F, Port C, and Port D.
// // - We just used Port B, Port F has only 5 exposed pins, Port C has 4, and D
// //   conveniently has 6 usable pins. So Port D, right?
// //    + Unfortunately, no. As mentioned above we can't actually use PD0 and PD1
// //      so we're at 4 usable pins for Port D and Port C.
// //    + Since we can't have matching number, we'll settle for matching physical
// //      locations: { A0..A5 } -> { C4, C5, C6, C7, D6, D7 }
// //    + Conveniently, the numbering isn't too awful either.
//
// For PWM:
//  - We need 2 pins.
//  - We've got lots of options (Ports B, C, D, F).
//  - Ultimately, I think it makes sense to go with Port F's LED pins so that
//    users can use them.
//  - We'll reserve Red for indicating hard faults so PF2 and PF3 it is!
//     + If we choose to have 4 PWM pins in the future we can do PF0 to PF3.
//
// This configuration leaves I2C, SPI, UART, QEI, and USB pins unused such that
// those peripherals can still be used, which is a nice bonus.
// // The Quadrature Encoder peripherals' pins don't share the same fate but alas.

struct Tm4cGpio<'a> {
    flags: Option<&'a GpioPinArr<AtomicBool>>,
}

impl<'a> Gpio<'a> for Tm4cGpio<'a> {
    fn set_state(&mut self, pin: GpioPin, state: GpioState) -> Result<(), GpioMiscError> {
        Err(GpioMiscError)
    }

    fn get_state(&self, pin: GpioPin) -> GpioState {
        GpioState::Disabled
    }

    fn read(&self, pin: GpioPin) -> Result<bool, GpioReadError> {
        Err(GpioReadError((pin, GpioState::Disabled)))
    }

    fn write(&mut self, pin: GpioPin, bit: bool) -> Result<(), GpioWriteError> {
        Err(GpioWriteError((pin, GpioState::Disabled)))
    }

    fn register_interrupt_flags(&mut self, flags: &'a GpioPinArr<AtomicBool>) {
        self.flags = Some(flags);
    }

    fn interrupt_occurred(&self, pin: GpioPin) -> bool {
        self.flags.unwrap()[pin].load(Ordering::SeqCst)
    }

    fn reset_interrupt_flag(&mut self, pin: GpioPin) {
        self.flags.unwrap()[pin].store(false, Ordering::SeqCst)
    }

    fn interrupts_enabled(&self, pin: GpioPin) -> bool {
        false
    }
}

impl<'a> Default for Tm4cGpio<'a> {
    fn default() -> Self {
        Self {
            flags: None,
        }
    }
}

use lc3_traits::peripherals::adc::{AdcPin, AdcState, AdcReadError/* , AdcPinArr */};

struct Tm4cAdc {

}

impl Adc for Tm4cAdc {
    fn set_state(&mut self, pin: AdcPin, state: AdcState) -> Result<(), ()> {
        Err(())
    }

    fn get_state(&self, pin: AdcPin) -> AdcState {
        AdcState::Disabled
    }

    fn read(&self, pin: AdcPin) -> Result<u8, AdcReadError> {
        Err(AdcReadError((pin, AdcState::Disabled)))
    }
}

impl Default for Tm4cAdc {
    fn default() -> Self { Self {} }
}

use lc3_traits::peripherals::pwm::{PwmState, PwmPin, PwmSetPeriodError, PwmSetDutyError, PwmPinArr};

struct Tm4cPwm {

}

impl Pwm for Tm4cPwm {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError> {
        Err(PwmSetPeriodError(pin))
    }

    fn get_state(&self, pin: PwmPin) -> PwmState {
        PwmState::Disabled
    }

    fn get_pin(&self, pin: PwmPin) -> bool {
        false
    }

    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError> {
        Err(PwmSetDutyError(pin))
    }

    fn get_duty_cycle(&self, pin: PwmPin) -> u8 {
        0
    }
}

impl Default for Tm4cPwm {
    fn default() -> Self { Self {} }
}

use lc3_traits::peripherals::timers::{TimerId, TimerMiscError, TimerState, TimerArr};

struct Tm4cTimers<'a> {
    flags: Option<&'a TimerArr<AtomicBool>>,
}

impl<'a> Timers<'a> for Tm4cTimers<'a> {
    fn set_state(&mut self, timer: TimerId, state: TimerState) -> Result<(), TimerMiscError> {
        Err(TimerMiscError)
    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        TimerState::Disabled
    }

    fn set_period(&mut self, timer: TimerId, ms: Word) -> Result<(), TimerMiscError> {
        Err(TimerMiscError)
    }

    fn get_period(&self, timer: TimerId) -> Word {
        0
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {
        self.flags = Some(flags)
    }

    fn interrupt_occurred(&self, timer: TimerId) -> bool {
        self.flags.unwrap()[timer].load(Ordering::SeqCst)
    }

    fn reset_interrupt_flag(&mut self, timer: TimerId) {
        self.flags.unwrap()[timer].store(false, Ordering::SeqCst)
    }

    fn interrupts_enabled(&self, tiner: TimerId) -> bool {
        false
    }
}

impl<'a> Default for Tm4cTimers<'a> {
    fn default() -> Self {
        Self {
            flags: None,
        }
    }
}

struct Tm4cClock {

}

impl Clock for Tm4cClock {
    fn get_milliseconds(&self) -> Word {
        0
    }

    fn set_milliseconds(&mut self, ms: Word) {
        asm::nop();
    }
}

impl Default for Tm4cClock {
    fn default() -> Self { Self { } }
}

use lc3_traits::peripherals::input::InputError;

struct Tm4cInput<'a> {
    flag: Option<&'a AtomicBool>,
    interrupts_enabled: bool,
    current_char: Cell<Option<u8>>,
}

impl<'a> Tm4cInput<'a> {
    fn fetch(&self) {
        let new_data: Option<u8> = None; // TODO!

        if let Some(c) = new_data {
            self.current_char.replace(Some(c));

            self.flag.unwrap().store(true, Ordering::SeqCst);
        }
    }
}

impl<'a> Input<'a> for Tm4cInput<'a> {
    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
        self.flag = Some(flag)
    }

    fn interrupt_occurred(&self) -> bool {
        self.current_data_unread()
    }

    fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupts_enabled = bit
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupts_enabled
    }

    fn read_data(&self) -> Result<u8, InputError> {
        self.fetch();

        self.flag.unwrap().store(false, Ordering::SeqCst);
        self.current_char.get().ok_or(InputError)
    }

    fn current_data_unread(&self) -> bool {
        self.fetch();

        self.flag.unwrap().load(Ordering::SeqCst)
    }
}

impl<'a> Default for Tm4cInput<'a> {
    fn default() -> Self {
        Self {
            flag: None,
            interrupts_enabled: false,
            current_char: Cell::new(None)
        }
    }
}

use lc3_traits::peripherals::output::OutputError;

struct Tm4cOutput<'a> {
    flag: Option<&'a AtomicBool>,
    interrupts_enabled: bool,
}

impl<'a> Output<'a> for Tm4cOutput<'a> {
    fn register_interrupt_flag(&mut self, flag: &'a AtomicBool) {
        self.flag = Some(flag);

        flag.store(true, Ordering::SeqCst)
    }

    fn interrupt_occurred(&self) -> bool {
        self.flag.unwrap().load(Ordering::SeqCst)
    }

    fn set_interrupt_enable_bit(&mut self, bit: bool) {
        self.interrupts_enabled = bit;
    }

    fn interrupts_enabled(&self) -> bool {
        self.interrupts_enabled
    }

    fn write_data(&mut self, c: u8) -> Result<(), OutputError> {
        self.flag.unwrap().store(false, Ordering::SeqCst);

        // TODO: write the character!

        self.flag.unwrap().store(true, Ordering::SeqCst);

        Ok(())
    }

    fn current_data_written(&self) -> bool {
        self.flag.unwrap().load(Ordering::SeqCst)
    }
}

impl<'a> Default for Tm4cOutput<'a> {
    fn default() -> Self {
        Self {
            flag: None,
            interrupts_enabled: false,
        }
    }
}
