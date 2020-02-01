//! Controller side for the [`Control`](super::Control) RPC set up.
//!
//! TODO!

// TODO: auto gen (proc macro, probably) the crimes below from the `Control`
// trait.

use super::{EventFutureSharedState, Control, Transport};
use super::messages::{RequestMessage, ResponseMessage};
use super::encoding::{Encode, Deocde, Transparent};
use crate::control::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use crate::control::{ProgramMetadata, DeviceInfo};
use crate::error::Error as Lc3Error;
use crate::memory::MemoryMiscError;
use crate::peripherals::{
    adc::{AdcPinArr, AdcState, AdcReadError},
    gpio::{GpioPinArr, GpioState, GpioReadError},
    pwm::{PwmPinArr, PwmState},
    timers::{TimerArr, TimerState},
};
use lc3_isa::{Reg, Addr, Word};

use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};


// Converts calls on the control interface to messages and sends said messages.
//
// Sends Requests and receives Responses.
//
// As mentioned elsewhere, there's a level of indirection between
// `RequestMessage`/`ResponseMessage` and the types used here so that users can
// experiment with their own message types. This is probably a moot point since
// you can already do this by defining an encoding layer that does the
// conversion for you. The only thing the below buys you is being able to use
// message types that don't implement Debug. (TODO)
#[derive(Debug, Default)]
pub struct Controller<
    'a,
    Req = RequestMessage,
    Resp = ResponseMessage,
    ReqEnc = Transparent,
    RespDec = Transparent,
    T, S
>
where
    Req: Into<RequestMessage>,
    Resp: Into<ResponseMessage>,
    ReqEnc: Encode<Req>,
    RespDec: Decode<Resp>,
    T: Transport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
    S: EventFutureSharedState,
{
    encoding: PhantomData<(ReqEnc, RespDec)>,
    pub transport: T,
    // pending_messages: Cell<[Option<ControlMessage>; 2]>,
    // pending_messages: [Option<ControlMessage>; 2],
    shared_state: &'a S,
    waiting_for_event: AtomicBool, // TODO: no reason for this to be Atomic // Note: it's atomic so we can maintain interior mutability?
    // waiting_for_event: bool,
}

// TODO: make a builder!

impl<'a, Req, Resp, E, D, T, S> Controller<'a, Req, Resp, E, D, T, S>
where
    Req: Into<RequestMessage>,
    Resp: Into<ResponseMessage>,
    E: Encode<Req>,
    D: Decode<Resp>,
    T: Transport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
    S: EventFutureSharedState,
{
    // When const functions can be in blanket impls, this can be made `const`.
    //
    // Note: we take `encoding` as a parameter here even though the actual value
    // is never used so that users don't have to resort to using the turbofish
    // syntax to specify what they want the encoding to be.
    pub /*const*/ fn new(_encoding: E, transport: T, shared_state: &'a S) -> Self {
        Self {
            // encoding,
            encoding: PhantomData,
            transport,
            // pending_messages: Cell::new([None; 2]),
            // pending_messages: [None; 2],
            shared_state,
            waiting_for_event: AtomicBool::new(false),
            // waiting_for_event: false,
        }
    }
}

impl<'a, Req, Resp, E, D, T, S> Controller<'a, Req, Resp, E, D, T, S>
where
    Req: Into<RequestMessage>,
    Resp: Into<ResponseMessage>,
    ReqEnc: Encode<Req>,
    RespDec: Decode<Resp>,
    T: Transport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
    S: EventFutureSharedState,
{
    // For now, we're going to assume sequential consistency (we receive
    // responses to messages in the same order we filed the requests). (TODO)
    //
    // Responses to our one non-blocking call (`run_until_event`) are the only
    // thing that could interrupt this.
    fn tick(&self) -> Option<Responses> {
        let encoded_message = self.transport.get()?;
        let message = E::decode(&encoded_message).unwrap(); // TODO: don't panic;
        let message = message.into();

        if let ResponseMessage::RunUntilEvent(event) = message {
            if self.waiting_for_event.load(Ordering::SeqCst) {
                // println!("resolving the rpc future"); // TODO: logging w/feature flag
                self.shared_state.resolve_all(event).unwrap();
                self.waiting_for_event.store(false, Ordering::SeqCst);

                None
            } else {
                // Something has gone very wrong.
                // We were told an event happened but we never asked.
                unreachable!()
            }
        } else {
            Some(message)
        }
    }
}


macro_rules! ctrl {
    ($s:ident, $req:expr, $resp:pat$(, $ret:expr)?) => {{
        use RequestMessage::*;
        use ResponseMessage as R;
        $s.transport.send(E::encode($req.into()).unwrap()).unwrap(); // TODO: don't panic

        loop {
            if let Some(m) = Controller::tick($s) {
                if let $resp = m {
                    break $($ret)?
                } else {
                    panic!("Incorrect response for message!")
                }
            }
        }
    }};
}

#[forbid(irrefutable_let_patterns)]
impl<'a, Req, Resp, E, D, T, S> Control for Controller<'a, Req, Resp, E, D, T, S>
where
    Req: Into<RequestMessage>,
    Resp: Into<ResponseMessage>,
    ReqEnc: Encode<Req>,
    RespDec: Decode<Resp>,
    T: Transport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
    S: EventFutureSharedState,
{
    type EventFuture = EventFuture<'a, S>;

    fn get_pc(&self) -> Addr { ctrl!(self, GetPc, R::GetPc(addr), addr) }
    fn set_pc(&mut self, addr: Addr) { ctrl!(self, SetPc { addr }, R::SetPc) }

    fn get_register(&self, reg: Reg) -> Word { ctrl!(self, GetRegister { reg }, R::GetRegister(word), word) }
    fn set_register(&mut self, reg: Reg, data: Word) { ctrl!(self, SetRegister { reg, data }, R::SetRegister) }

    fn get_registers_psr_and_pc(&self) -> ([Word; Reg::NUM_REGS], Word, Word) {
        ctrl!(self, GetRegistersPsrAndPc, R::GetRegistersPsrAndPc(r), r)
    }

    fn read_word(&self, addr: Addr) -> Word { ctrl!(self, ReadWord { addr }, R::ReadWord(w), w) }
    fn write_word(&mut self, addr: Addr, word: Word) { ctrl!(self, WriteWord { addr, word }, R::WriteWord) }
    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        ctrl!(self, CommitMemory, R::CommitMemory(r), r)
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        ctrl!(self, SetBreakpoint { addr }, R::SetBreakpoint(r), r)
    }
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        ctrl!(self, UnsetBreakpoint { idx }, R::UnsetBreakpoint(r), r)
    }
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] { ctrl!(self, GetBreakpoints, R::GetBreakpoints(r), r) }
    fn get_max_breakpoints(&self) -> usize { ctrl!(self, GetMaxBreakpoints, R::GetMaxBreakpoints(r), r) }

    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        ctrl!(self, SetMemoryWatchpoint { addr }, R::SetMemoryWatchpoint(r), r)
    }
    fn unset_memory_watchpoint(&mut self, idx: usize) -> Result<(), ()> {
        ctrl!(self, UnsetMemoryWatchpoint { idx }, R::UnsetBreakpoint(r), r)
    }
    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS] { ctrl!(self, GetMemoryWatchpoints, R::GetMemoryWatchpoints(r), r) }
    fn get_max_memory_watchpoints(&self) -> usize { ctrl!(self, GetMaxMemoryWatchpoints, R::GetMaxMemoryWatchpoints(r), r) }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {
        // If we're in a sealed batch with pending futures, just crash.
        self.shared_state.add_new_future().expect("no new futures once a batch starts to resolve");

        // If we're already waiting for an event, don't bother sending the
        // request along again:
        if !self.waiting_for_event.load(Ordering::SeqCst) {
            self.transport.send(E::encode(/*RequestMessage::*/RunUntilEvent.into()).unwrap()).unwrap();

            // Wait for the acknowledge:
            loop {
                if let Some(m) = Controller::tick(self) {
                    if let ResponseMessage::RunUntilEventAck = m {
                        break;
                    } else {
                        panic!("Incorrect response for message!")
                    }
                }
            }

            self.waiting_for_event.store(true, Ordering::SeqCst);
        }

        // println!("new rpc future");

        EventFuture(self.shared_state)
    }

    fn tick(&mut self) {
        // Because we basically call tick() on every other function call here, we could
        // probably get away with just doing nothing here in practice.
        // But, checking here as well doesn't hurt.
        //
        // We should never actually get a message here (run until event responses are
        // handled within `Self::tick()`) though.
        // Self::tick(self).unwrap_none(); // when this goes stable, use this, maybe (TODO)
        if Self::tick(self).is_some() { panic!("Controller received a message in tick!") }
    }

    fn step(&mut self) -> Option<Event> { ctrl!(self, Step, R::Step(r), r) }
    fn pause(&mut self) { ctrl!(self, Pause, R::Pause) }

    fn get_state(&self) -> State { ctrl!(self, GetState, R::GetState(r), r) }

    fn reset(&mut self) {
        // Drop all pending futures:
        // (if one of these futures is polled again, bad bad things will happen; TODO)
        self.shared_state.reset();

        ctrl!(self, Reset, R::Reset)
    }

    fn get_error(&self) -> Option<Lc3Error> { ctrl!(self, GetError, R::GetError(r), r) }

    // I/O Access:
    fn get_gpio_states(&self) -> GpioPinArr<GpioState> { ctrl!(self, GetGpioStates, R::GetGpioStates(r), r) }
    fn get_gpio_readings(&self) -> GpioPinArr<Result<bool, GpioReadError>> { ctrl!(self, GetGpioReadings, R::GetGpioReadings(r), r) }
    fn get_adc_states(&self) -> AdcPinArr<AdcState> { ctrl!(self, GetAdcStates, R::GetAdcStates(r), r) }
    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>> { ctrl!(self, GetAdcReadings, R::GetAdcReadings(r), r) }
    fn get_timer_states(&self) -> TimerArr<TimerState> { ctrl!(self, GetTimerStates, R::GetTimerStates(r), r) }
    fn get_timer_config(&self) -> TimerArr<Word> { ctrl!(self, GetTimerConfig, R::GetTimerConfig(r), r) }
    fn get_pwm_states(&self) -> PwmPinArr<PwmState> { ctrl!(self, GetPwmStates, R::GetPwmStates(r), r) }
    fn get_pwm_config(&self) -> PwmPinArr<u8> { ctrl!(self, GetPwmConfig, R::GetPwmConfig(r), r) }
    fn get_clock(&self) -> Word { ctrl!(self, GetClock, R::GetClock(r), r) }

    fn get_info(&self) -> DeviceInfo { ctrl!(self, GetInfo, R::GetInfo(r), r) }
    fn set_program_metadata(&mut self, metadata: ProgramMetadata) { ctrl!(self, ProgramMetadata { metadata }, R::ProgramMetadata) }
}
