//! The [`Control` trait](crate::control::Control) and friends.
//!
//! Unlike the [`Peripherals` trait](crate::peripherals::Peripherals) and the
//! [`Memory` trait](crate::memory::Memory), there is no shim implementation of
//! Control; instead the 'shim' is an instruction level simulator that lives in
//! the [interp module](crate::interp).
//!
//! TODO!

use crate::error::Error;
use crate::peripherals::adc::{AdcPinArr, AdcReadError, AdcState};
use crate::peripherals::gpio::{GpioPinArr, GpioReadError, GpioState};
use crate::peripherals::pwm::{PwmPinArr, PwmState};
use crate::peripherals::timers::{TimerArr, TimerState};
use super::{Capabilities, DeviceInfo, ProgramMetadata, Identifier, Verion};
use super::load::{PageIndex, PageWriteStart, StartPageWriteError, PageChunkError, FinishPageWriteError, LoadApiSession, Offset, CHUNK_SIZE_IN_WORDS};

use lc3_isa::{Addr, Reg, Word, PSR};

use core::future::Future;

use serde::{Deserialize, Serialize};

pub const MAX_BREAKPOINTS: usize = 10;
pub const MAX_MEMORY_WATCHPOINTS: usize = 10;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    Breakpoint { addr: Addr },
    MemoryWatch { addr: Addr, data: Word },
    Error { err: Error },
    Interrupted, // If we get paused or stepped, this is returned. (TODO: we currently only return this if we're paused!! not sure if stopping on a step is reasonable behavior)
    Halted,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum State {
    Paused,
    RunningUntilEvent,
    Halted,
}

// Actually maybe make this Control a super trait of this can have Control still retain
// EventFuture and run_until_event. (TODO)
// pub trait Driver: Control { type EventFuture:
// Future<Output = (Event, State)>;

//     fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.

//     fn make_progress(&mut self);
// }

pub trait Control {
    type EventFuture: Future<Output = Event>;

    fn get_pc(&self) -> Addr;
    fn set_pc(&mut self, addr: Addr); // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word;
    fn set_register(&mut self, reg: Reg, data: Word); // Should be infallible.

    fn get_registers_psr_and_pc(&self) -> ([Word; Reg::NUM_REGS], Word, Word) {
        let mut regs = [0; Reg::NUM_REGS];

        Reg::REGS
            .iter()
            .enumerate()
            .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

        (regs, self.read_word(PSR), self.get_pc())
    }

    fn read_word(&self, addr: Addr) -> Word;
    fn write_word(&mut self, addr: Addr, word: Word);

    /// The start function for a Load API Session.
    ///
    /// Calling this is effectively unsafe since you need to call [an unsafe
    /// function](crate::control::load::LoadApiSession<PageWriteStart>::new) to
    /// construct the `page` token.
    ///
    /// Unless you have a special use case, you should use
    /// [`load_memory_dump`](crate::control::load::load_memory_dump) and its
    /// fellow functions in the [`load` module](crate::control::load) or the
    /// [`load` function](lc3_shims::memory::FileBackedMemoryShim::load) on the
    /// [file backed `Memory` shim](lc3_shims::memory::FileBackedMemoryShim).
    fn start_page_write(
        &mut self,
        page: LoadApiSession<PageWriteStart>,
        checksum: u64,
    ) -> Result<LoadApiSession<PageIndex>, StartPageWriteError>;

    /// The workhorse of the Load API. Sends a [single chunk](chunk).
    ///
    /// Note that this only takes an offset into the current session's page
    /// instead of a full address; because the only (sans unsafe) way to
    /// construct this offset is to use the [`with_offset` function](wo_func) on
    /// the `LoadApiSession<PageIndex>` returned by the
    /// [start_page_write function](start).
    ///
    /// [chunk]: crate::control::load::CHUNK_SIZE_IN_WORDS
    /// [wo_func]: crate::control::load::LoadApiSession<PageIndex>::with_offset
    /// [start]: crate::control::control::Control::start_page_write
    fn send_page_chunk(
        &mut self,
        offset: LoadApiSession<Offset>,
        chunk: [Word; CHUNK_SIZE_IN_WORDS as usize],
    ) -> Result<(), PageChunkError>;

    /// The finish function for a Load API Session.
    ///
    /// Consumes the `LoadApiSession<PageIndex>` returned by
    /// [`start_page_write`](start) and makes it impossible to call
    /// [`send_page_chunk`](send) again without starting a new session, thus
    /// ending the session.
    ///
    /// [start]: crate::control::control::Control::start_page_write
    /// [send]: crate::control::control::Control::send_page_chunk
    fn finish_page_write(
        &mut self,
        page: LoadApiSession<PageIndex>,
    ) -> Result<(), FinishPageWriteError>;

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>;
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS];
    fn get_max_breakpoints(&self) -> usize {
        MAX_BREAKPOINTS
    }

    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    fn unset_memory_watchpoint(&mut self, idx: usize) -> Result<(), ()>;
    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS];
    fn get_max_memory_watchpoints(&self) -> usize {
        MAX_MEMORY_WATCHPOINTS
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.
    // TODO: we probably want a better API than this...
    // Maybe a Driver trait that takes a FnMut(impl Control)
    // that calls tick under the hood and then the function provided

    // The invariant to maintain is that if run_until_event is called, tick must be
    // called periodically.
    //
    // Also, this function will *not* be proxied.
    //
    // Returns the number of instructions executed. This will be used to know whether
    // or not it is critical that this function is still called regularly.
    //
    // This is allowed to be an estimate, so long as the following invariant is
    // maintained:
    //   - if one or more instructions was executed, this must return a number greater
    //     than 0.
    fn tick(&mut self) -> usize; // The function to call so that the simulator can do some work.

    fn step(&mut self) -> Option<Event>;
    fn pause(&mut self); // TODO: should we respond saying whether or not the pause actually did anything (i.e. if we were already paused... it did not).

    fn get_state(&self) -> State;

    fn reset(&mut self); // Note: needs to reset memory!

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error>;

    // I/O Access:
    // TODO!! Does the state/reading separation make sense?
    fn get_gpio_states(&self) -> GpioPinArr<GpioState>;
    fn get_gpio_readings(&self) -> GpioPinArr<Result<bool, GpioReadError>>;
    fn get_adc_states(&self) -> AdcPinArr<AdcState>;
    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>>;
    fn get_timer_modes(&self) -> TimerArr<TimerMode>;
    fn get_timer_states(&self) -> TimerArr<TimerState>;
    fn get_pwm_states(&self) -> PwmPinArr<PwmState>;
    fn get_pwm_config(&self) -> PwmPinArr<u8>; // TODO: ditto with using u8 here; probably should be some kind of enum (the conflict is then we're kinda pushing implementors to represent state a certain way.. or at least to have to translate it to our enum).
    fn get_clock(&self) -> Word;

    // So with some of these functions that are basically straight wrappers over their Memory/Peripheral trait counterparts,
    // we have a bit of a choice. We can make Control a super trait of those traits so that we can have default impls of said
    // functions or we can make the implementor of Control manually wrap those functions.
    //
    // The downside to the super trait way is that it's a little weird; it requires that one massive type hold all the state
    // for all the Peripherals and Memory (and whatever the impl for Control ends up needing). You can of course store the
    // state for those things in their own types within your big type, but then to impl, say, Memory, you'd have to manually
    // pass all the calls along meaning we're back where we started.
    //
    // Associated types really don't seem to save us here (still gotta know where the state is stored which we don't know
    // when writing a default impl) and I can't think of a way that's appreciably better so I think we just have to eat it.
    //
    // We kind of got around this with the `PeripheralSet` struct in the peripherals module, but I'm not sure it'd work here.

    // We encourage implementors to not return different device names,
    // capabilities, and versions dynamically _unless they have a good reason
    // for doing so_ (i.e. attaching an SD Card to a particular implementation
    // enables the disk peripheral).
    fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo::new(
            self.id(),
            super::version_from_crate!(),
            core::any::TypeId::of::<()>(),
            Capabilities::default(),
            Default::default() // (no proxies by default)
        )
    }

    fn get_program_metadata(&self) -> ProgramMetadata;
    fn set_program_metadata(&mut self, metadata: ProgramMetadata);

    // Should actually be an associated constant but isn't because of object
    // safety.
    //
    // As such, this function isn't proxied.
    #[doc(hidden)]
    fn id(&self) -> Identifier {
        Identifier::new_from_str_that_crashes_on_invalid_inputs("????")
    }
}
