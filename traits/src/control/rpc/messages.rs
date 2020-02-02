//! Messages used for proxying [Control trait](super::Control) functions.

use crate::error::Error as Lc3Error;
use crate::peripherals::{adc::AdcPinArr, gpio::GpioPinArr, pwm::PwmPinArr, timers::TimerArr}
use crate::peripherals::{adc::AdcReadError, gpio::GpioReadError};
use crate::memory::MemoryMiscError;
use crate::control::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use crate::control::metadata::{DeviceInfo, ProgramMetadata};
use super::{State, Event};

use lc3_isa::{Addr, Reg, Word};

// TODO: auto gen (proc macro, probably) the types below from and the `Control`
// trait.

// TODO: one strategy to reduce message enum sizes is to not proxy the
// "convenience calls" (i.e. the ones that Control has default impls for like
// `get_register_psr_and_pc`) and instead have the proxying things use the
// default impls too and to just be okay with one call on the controller side
// for these messages resulting in multiple messages being sent.
//
// The benefit would be not needing to have dedicated messages for these things
// so a smaller message size (I'm assuming that these variants are the largest
// ones).
//
// On the other hand decent compression in the encoding layer would achieve
// basically the same thing (assuming that I/O throughput is the bottleneck)
// without adding more message related overhead to these "convenience calls".

#[allow(dead_code)]
static __REQ_SIZE_CHECK: () = {
    let s = core::mem::size_of::<RequestMessage>();
    let canary = [()];

    canary[s - 32] // panic if the size of RequestMessage changes
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[deny(clippy::large_enum_variant)]
pub enum RequestMessage { // messages for everything but tick()
    GetPc,
    SetPc { addr: Addr },

    GetRegister { reg: Reg },
    SetRegister { reg: Reg, data: Word },

    // Optional, but we're including it in case implementors wish to do
    // something special or just cut down on overhead.
    GetRegistersPsrAndPc,

    ReadWord { addr: Addr },
    WriteWord { addr: Addr, word: Word },
    CommitMemory,

    SetBreakpoint { addr: Addr },
    UnsetBreakpoint { idx: usize },
    GetBreakpoints,
    GetMaxBreakpoints,

    SetMemoryWatchpoint { addr: Addr },
    UnsetMemoryWatchpoint { idx: usize },
    GetMemoryWatchpoints,
    GetMaxMemoryWatchpoints,

    // no tick!
    RunUntilEvent,

    Step,
    Pause,

    GetState,

    Reset,

    GetError,

    GetGpioStates,
    GetGpioReadings,
    GetAdcStates,
    GetAdcReadings,
    GetTimerStates,
    GetTimerConfig,
    GetPwmStates,
    GetPwmConfig,
    GetClock,

    GetInfo,
    SetProgramMetadata { metadata: ProgramMetadata },
}

#[allow(dead_code)]
static __RESP_SIZE_CHECK: () = {
    let s = core::mem::size_of::<ResponseMessage>();
    let canary = [()];

    canary[s - 72] // panic if the size of ResponseMessage changes
};

#[derive(Debug, Clone, Serialize, Deserialize, Debug)]
#[deny(clippy::large_enum_variant)]
pub enum ResponseMessage { // messages for everything but tick()
    GetPc(Addr),
    SetPc,

    GetRegister(Word),
    SetRegister,

    // Optional, but we're including it in case implementors wish to do
    // something special or just cut down on overhead.
    GetRegistersPsrAndPc(([Word; Reg::NUM_REGS], Word, Word)),

    ReadWord(Word),
    WriteWord,
    CommitMemory(Result<(), MemoryMiscError>),

    SetBreakpoint(Result<usize, ()>),
    UnsetBreakpoint(Result<(), ()>),
    GetBreakpoints([Option<Addr>; MAX_BREAKPOINTS]),
    GetMaxBreakpoints(usize),

    SetMemoryWatchpoint(Result<usize, ()>),
    UnsetMemoryWatchpoint(Result<(), ()>),
    GetMemoryWatchpoints([Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS]),
    GetMaxMemoryWatchpoints(usize),

    // no tick!
    RunUntilEventAck, // Special acknowledge message for run until event.
    RunUntilEvent(Event),

    Step(Option<Event>),
    Pause,

    GetState(State),
    Reset,

    GetError(Option<Lc3Error>),

    GetGpioStates(GpioPinArr<GpioState>),
    GetGpioReadings(GpioPinArr<Result<bool, GpioReadError>>),
    GetAdcStates(AdcPinArr<AdcState>),
    GetAdcReadings(AdcPinArr<Result<u8, AdcReadError>>),
    GetTimerStates(TimerArr<TimerState>),
    GetTimerConfig(TimerArr<Word>), // TODO
    GetPwmStates(PwmPinArr<PwmState>),
    GetPwmConfig(PwmPinArr<u8>), // TODO
    GetClock(Word),

    GetInfo(DeviceInfo),
    SetProgramMetadata,
}
