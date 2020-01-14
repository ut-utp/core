//! Messages used for proxying [Control trait](super::Control) functions.

#[allow(dead_code)]
static FOO: () = {
    let s = core::mem::size_of::<ControlMessage>();
    let canary = [()];

    canary[s - 64] // panic if the size of ControlMessage changes
};

// TODO: split into request/response types (helps with type safety (i.e. Device only
// deals with Responses) and potentially the size of the messages)
#[derive(Clone, Serialize, Deserialize, Debug)]
#[deny(clippy::large_enum_variant)]
pub enum ControlMessage { // messages for everything but tick()
    GetPcRequest,
    GetPcResponse(Addr),

    SetPcRequest { addr: Addr },
    SetPcSuccess,

    GetRegisterRequest { reg: Reg },
    GetRegisterResponse(Word),

    SetRegisterRequest { reg: Reg, data: Word },
    SetRegisterSuccess,

    // Optional, but we're including it in case implementors wish to do
    // something special or just cut down on overhead.
    GetRegistersPsrAndPcRequest,
    GetRegistersPsrAndPcResponse(([Word; Reg::NUM_REGS], Word, Word)),

    ReadWordRequest { addr: Addr },
    ReadWordResponse(Word),

    WriteWordRequest { addr: Addr, word: Word },
    WriteWordSuccess,

    CommitMemoryRequest,
    CommitMemoryResponse(Result<(), MemoryMiscError>),

    SetBreakpointRequest { addr: Addr },
    SetBreakpointResponse(Result<usize, ()>),

    UnsetBreakpointRequest { idx: usize },
    UnsetBreakpointResponse(Result<(), ()>),

    GetBreakpointsRequest,
    GetBreakpointsResponse([Option<Addr>; MAX_BREAKPOINTS]),

    GetMaxBreakpointsRequest,
    GetMaxBreakpointsResponse(usize),

    SetMemoryWatchpointRequest { addr: Addr },
    SetMemoryWatchpointResponse(Result<usize, ()>),

    UnsetMemoryWatchpointRequest { idx: usize },
    UnsetMemoryWatchpointResponse(Result<(), ()>),

    GetMemoryWatchpointsRequest,
    GetMemoryWatchpointsResponse([Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS]),

    GetMaxMemoryWatchpointsRequest,
    GetMaxMemoryWatchpointsResponse(usize),

    // (TODO)
    RunUntilEventRequest,
    RunUntilEventResponse(Event),
    // TODO: add a quick immediate response message (probably should do this!)
    // (call it success!)

    StepRequest,
    StepResponse(Option<Event>),

    PauseRequest,
    PauseSuccess,

    GetStateRequest,
    GetStateResponse(State),

    ResetRequest,
    ResetSuccess,

    // (TODO)
    GetErrorRequest,
    GetErrorResponse(Option<Lc3Error>),

    GetGpioStatesRequest,
    GetGpioStatesResponse(GpioPinArr<GpioState>),

    GetGpioReadingsRequest,
    GetGpioReadingsResponse(GpioPinArr<Result<bool, GpioReadError>>),

    GetAdcStatesRequest,
    GetAdcStatesResponse(AdcPinArr<AdcState>),

    GetAdcReadingsRequest,
    GetAdcReadingsResponse(AdcPinArr<Result<u8, AdcReadError>>),

    GetTimerStatesRequest,
    GetTimerStatesResponse(TimerArr<TimerState>),

    GetTimerConfigRequest,
    GetTimerConfigResponse(TimerArr<Word>), // TODO

    GetPwmStatesRequest,
    GetPwmStatesResponse(PwmPinArr<PwmState>),

    GetPwmConfigRequest,
    GetPwmConfigResponse(PwmPinArr<u8>), // TODO

    GetClockRequest,
    GetClockResponse(Word),
}
