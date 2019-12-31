// #![cfg_attr(not(test), no_std)]
#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use core::sync::atomic::{AtomicBool, Ordering};

use lc3_isa::{Addr, Word};
use lc3_traits::memory::{Memory, MemoryMiscError};

/// We've got limited space, so here's what we'll do for now.
/// 256 Word (i.e. 512 byte) pages.
///
///   0: 0x0000 - 0x00FF :: backed (vectors)
///   1: 0x0100 - 0x01FF :: backed (vectors)
///   2: 0x0200 - 0x02FF :: backed (OS)
///   3: 0x0300 - 0x03FF :: backed (OS)
///   4: 0x0400 - 0x04FF :: backed (config)
/// ..........................................
///  47: 0x2F00 - 0x2FFF :: backed (OS stack)
///  48: 0x3000 - 0x30FF :: backed (user prog)
///  49: 0x3100 - 0x31FF :: backed (user prog)
///  50: 0x3200 - 0x32FF :: backed (user prog)
///  51: 0x3300 - 0x33FF :: backed (user prog)
///  52: 0x3400 - 0x34FF :: backed (user prog)
///  53: 0x3500 - 0x35FF :: backed (user prog)
///  54: 0x3600 - 0x36FF :: backed (user prog)
///  55: 0x3700 - 0x37FF :: backed (user prog)
///  56: 0x3800 - 0x38FF :: backed (user prog)
///  57: 0x3900 - 0x39FF :: backed (user prog)
///  58: 0x3A00 - 0x3AFF :: backed (user prog)
///  59: 0x3B00 - 0x3BFF :: backed (user prog)
///  60: 0x3C00 - 0x3CFF :: backed (user prog)
///  61: 0x3D00 - 0x3DFF :: backed (user prog)
///  62: 0x3E00 - 0x3EFF :: backed (user prog)
///  63: 0x3F00 - 0x3FFF :: backed (user prog)
/// ..........................................
/// 254: 0xFE00 - 0xFEFF :: backed (mem mapped special)
/// 255: 0xFF00 - 0xFFFF :: backed (mem mapped special)
///
/// 24 of these pages will occupy 12KiB of RAM, which we should be able to
/// handle.
///
struct Tm4cMemory {
    pages: [[Word; Self::PAGE_SIZE]; 24],
    zero: Word,
    void: Word,
}

impl Tm4cMemory {
    const PAGE_SIZE: usize = 0x0100;

    fn addr_to_page(addr: Addr) -> Option<(usize, usize)> {
        let offset: usize = (addr as usize) % Self::PAGE_SIZE;

        match addr {
            0x0000..=0x04FF => Some(((addr as usize / Self::PAGE_SIZE), offset)),
            0x2F00..=0x3FFF => Some(((addr as usize / Self::PAGE_SIZE) - 0x2F + 5, offset)),
            0xFE00..=0xFEFF => Some(((addr as usize / Self::PAGE_SIZE) - 0xFE + 22, offset)),
            _ => None,
        }
    }
}

impl Index<Addr> for Tm4cMemory {
    type Output = Word;

    fn index(&self, addr: Addr) -> &Self::Output {
        match Tm4cMemory::addr_to_page(addr) {
            Some((page, offset)) => {
                &self.pages[page][offset]
            },
            None => &self.zero,
        }
    }
}

impl IndexMut<Addr> for Tm4cMemory {
    fn index_mut(&mut self, addr: Addr) -> &mut Self::Output {
        match Tm4cMemory::addr_to_page(addr) {
            Some((page, offset)) => {
                &mut self.pages[page][offset]
            },
            None => {
                self.void = 0;
                &mut self.void
            },
        }
    }
}

impl Default for Tm4cMemory {
    fn default() -> Self {
        Self {
            pages: [[0; Tm4cMemory::PAGE_SIZE]; 24],
            zero: 0,
            void: 0,
        }
    }
}

impl Memory for Tm4cMemory {
    fn commit(&mut self) -> Result<(), MemoryMiscError> {
        Err(MemoryMiscError) // No persistent storage for now!
    }
}

use lc3_traits::peripherals::{Gpio, Adc, Pwm, Timers, Clock, Input, Output};

use lc3_traits::peripherals::gpio::{GpioPin, GpioState, GpioPinArr, GpioReadError, GpioWriteError, GpioMiscError};

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

use lc3_traits::control::rpc::SimpleEventFutureSharedState;

// static SHARED_STATE: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

use lc3_baseline_sim::interp::{Interpreter, InstructionInterpreter, InterpreterBuilder, PeripheralInterruptFlags};
use lc3_baseline_sim::sim::Simulator;

use lc3_traits::peripherals::PeripheralSet;
use lc3_traits::control::Control;

#[entry]
fn main() -> ! {
    type Interp<'a> = Interpreter<'a,
        Tm4cMemory,
        PeripheralSet<'a,
            Tm4cGpio<'a>,
            Tm4cAdc,
            Tm4cPwm,
            Tm4cTimers<'a>,
            Tm4cClock,
            Tm4cInput<'a>,
            Tm4cOutput<'a>,
        >
    >;

    let flags: PeripheralInterruptFlags = PeripheralInterruptFlags::new();

    let state: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

    let mut interp: Interp = InterpreterBuilder::new()
        .with_defaults()
        .build();

    interp.reset();
    interp.init(&flags);

    let mut sim = Simulator::new_with_state(interp, &state);
    sim.reset();

    loop {
        // your code goes here
        sim.step();
    }
}
