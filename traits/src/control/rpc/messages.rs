//! Messages used for proxying [Control trait](super::Control) functions.

use super::{State, Event};
use crate::control::control::{
    MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS, MAX_CALL_STACK_DEPTH
};
use crate::control::load::{
    LoadApiSession, CHUNK_SIZE_IN_WORDS, PageWriteStart, PageIndex, Offset,
    StartPageWriteError, PageChunkError, FinishPageWriteError
};
use crate::control::{ProgramMetadata, DeviceInfo, UnifiedRange, ProcessorMode, Idx};
use crate::error::Error as Lc3Error;
use crate::peripherals::{
    adc::{AdcPinArr, AdcState, AdcReadError},
    gpio::{GpioPinArr, GpioState, GpioReadError},
    pwm::{PwmPinArr, PwmState},
    timers::{TimerArr, TimerMode, TimerState},
};

use lc3_isa::{Addr, Reg, Word};

use serde::{Serialize, Deserialize};

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
// We're not using static_assertions here so that we can get an error that tells
// us how much we're off by.
static __REQ_SIZE_CHECK: () = {
    let canary = [()];

    canary[REQUEST_MESSAGE_SIZE - 40] // panic if the size of RequestMessage changes
};

pub const REQUEST_MESSAGE_SIZE: usize = core::mem::size_of::<RequestMessage>();

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    StartPageWrite { page: LoadApiSession<PageWriteStart>, checksum: u64 },
    SendPageChunk { offset: LoadApiSession<Offset>, chunk: [Word; CHUNK_SIZE_IN_WORDS as usize] },
    FinishPageWrite { page: LoadApiSession<PageIndex> },

    SetBreakpoint { addr: Addr },
    UnsetBreakpoint { idx: Idx },
    GetBreakpoints,
    GetMaxBreakpoints,

    SetMemoryWatchpoint { addr: Addr },
    UnsetMemoryWatchpoint { idx: Idx },
    GetMemoryWatchpoints,
    GetMaxMemoryWatchpoints,

    SetDepthCondition { condition: UnifiedRange<u64> },
    UnsetDepthCondition,
    GetDepth,
    GetCallStack,

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
    GetTimerModes,
    GetTimerStates,
    GetPwmStates,
    GetPwmConfig,
    GetClock,

    GetDeviceInfo,

    GetProgramMetadata,
    SetProgramMetadata { metadata: ProgramMetadata },

    // no id!
}

#[allow(dead_code)]
static __RESP_SIZE_CHECK: () = {
    let canary = [()];

    canary[RESPONSE_MESSAGE_SIZE - 72] // panic if the size of ResponseMessage changes
};

pub const RESPONSE_MESSAGE_SIZE: usize = core::mem::size_of::<ResponseMessage>();

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    StartPageWrite(Result<LoadApiSession<PageIndex>, StartPageWriteError>),
    SendPageChunk(Result<(), PageChunkError>),
    FinishPageWrite(Result<(), FinishPageWriteError>),

    SetBreakpoint(Result<Idx, ()>),
    UnsetBreakpoint(Result<(), ()>),
    GetBreakpoints([Option<Addr>; MAX_BREAKPOINTS]),
    GetMaxBreakpoints(Idx),

    SetMemoryWatchpoint(Result<Idx, ()>),
    UnsetMemoryWatchpoint(Result<(), ()>),
    GetMemoryWatchpoints([Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS]),
    GetMaxMemoryWatchpoints(Idx),

    SetDepthCondition(Result<Option<UnifiedRange<u64>>, ()>),
    UnsetDepthCondition(Option<UnifiedRange<u64>>),
    GetDepth(Result<u64, ()>),
    GetCallStack([Option<(Addr, ProcessorMode)>; MAX_CALL_STACK_DEPTH]),

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
    GetTimerModes(TimerArr<TimerMode>),
    GetTimerStates(TimerArr<TimerState>),
    GetPwmStates(PwmPinArr<PwmState>),
    GetPwmConfig(PwmPinArr<u8>), // TODO
    GetClock(Word),

    GetDeviceInfo(DeviceInfo),

    GetProgramMetadata(ProgramMetadata),
    SetProgramMetadata,

    // no id!
}


// This workaround allows us to avoid having a Clone impl on RequestMessage and
// ResponseMessage which allows us to avoid having a Clone impl on
// LoadApiSession which makes it harder to misuse the Load API.
//
// The absence of a Clone impl for LoadApiSession does not close all the holes;
// it's still possible to 'clone' it using Serialize/Deserialize.
//
// This was going to be the workaround for implementing Transparent for Req/Resp
// below but rather than go through the trouble of creating our own no-op serde
// Serializer and Deserializer, we just implement Clone (using unsafe). The
// reasoning here is that there are already holes in LoadApiSession and being
// able to clone a Request or Response Message with a LoadApiSession instance in
// it isn't really a problem since people outside of this crate can't actually
// extract the fields of Req/Resp messages.
//
// As for the use of unsafe: LoadApiSession is the only thing in the type that
// doesn't impl Clone and this is not for memory safety reasons.
//
// In case this is not true for fields that are added later (and also because
// not everything in the above impls Copy) we match on the variants and
// clone/copy them manually.

use core::mem::MaybeUninit;
use core::ptr::copy;

unsafe fn force_clone<T>(inp: &T) -> T {
    let mut out: MaybeUninit<T> = MaybeUninit::uninit();

    #[allow(unsafe_code, unused_unsafe)]
    // This isn't universally safe; the caller has to promise that T is really
    // a Copy type (or _could be_ a Copy type).
    unsafe { copy(inp as *const _, out.as_mut_ptr(), 1); }

    #[allow(unsafe_code, unused_unsafe)]
    // This is safe since we just wrote to all the bits in T when we did the
    // copy above.
    unsafe { out.assume_init() }
}

impl Clone for RequestMessage {
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    #[inline]
    fn clone(&self) -> Self {
        use RequestMessage::*;
        macro_rules! variants {
            ($(
                $nom:tt $({ $($fields:ident),* })?
            ),*) => {
                match self {
                    $(
                        $nom $({ $($fields),* })? => $nom $({ $($fields: $fields.clone()),* })?,
                    )*
                    StartPageWrite { page, checksum } => {
                        StartPageWrite {
                            #[allow(unsafe_code)]
                            page: unsafe { force_clone(page) },
                            checksum: *checksum,
                        }
                    },
                    SendPageChunk { offset, chunk } => {
                        SendPageChunk {
                            #[allow(unsafe_code)]
                            offset: unsafe { force_clone(offset) },
                            chunk: chunk.clone(),
                        }
                    },
                    FinishPageWrite { page } => {
                        FinishPageWrite {
                            #[allow(unsafe_code)]
                            page: unsafe { force_clone(page) }
                        }
                    }
                }
            };
        }

        variants! {
            GetPc,
            SetPc { addr },
            GetRegister { reg },
            SetRegister { reg, data },
            GetRegistersPsrAndPc,
            ReadWord { addr },
            WriteWord { addr, word },
            SetBreakpoint { addr },
            UnsetBreakpoint { idx },
            GetBreakpoints,
            GetMaxBreakpoints,
            SetMemoryWatchpoint { addr },
            UnsetMemoryWatchpoint { idx },
            GetMemoryWatchpoints,
            GetMaxMemoryWatchpoints,
            SetDepthCondition { condition },
            UnsetDepthCondition,
            GetDepth,
            GetCallStack,
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
            GetTimerModes,
            GetTimerStates,
            GetPwmStates,
            GetPwmConfig,
            GetClock,
            GetDeviceInfo,
            GetProgramMetadata,
            SetProgramMetadata { metadata }
        }
    }
}

impl Clone for ResponseMessage {
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    #[inline]
    fn clone(&self) -> Self {
        use ResponseMessage::*;
        macro_rules! variants {
            ($(
                $nom:tt $(( $($fields:ident),* ))?
            ),*) => {
                match self {
                    $(
                        $nom $(( $($fields),* ))? => $nom $(( $($fields.clone()),* ))?,
                    )*
                    StartPageWrite(r) => {
                        #[allow(unsafe_code)]
                        StartPageWrite(unsafe { force_clone(r) })
                    }
                }
            };
        }

        variants! {
            GetPc(a),
            SetPc,
            GetRegister(w),
            SetRegister,
            GetRegistersPsrAndPc(t),
            ReadWord(w),
            WriteWord,
            SetBreakpoint(r),
            UnsetBreakpoint(r),
            GetBreakpoints(bps),
            GetMaxBreakpoints(i),
            SetMemoryWatchpoint(r),
            UnsetMemoryWatchpoint(r),
            GetMemoryWatchpoints(wps),
            GetMaxMemoryWatchpoints(i),
            SetDepthCondition(r),
            UnsetDepthCondition(r),
            GetDepth(r),
            GetCallStack(s),
            RunUntilEventAck,
            RunUntilEvent(e),
            Step(e),
            Pause,
            GetState(s),
            Reset,
            GetError(e),
            GetGpioStates(s),
            GetGpioReadings(r),
            GetAdcStates(s),
            GetAdcReadings(r),
            GetTimerModes(m),
            GetTimerStates(s),
            GetPwmStates(s),
            GetPwmConfig(c),
            GetClock(w),
            GetDeviceInfo(i),
            GetProgramMetadata(m),
            SetProgramMetadata,

            SendPageChunk(r),
            FinishPageWrite(r)
        }
    }
}
