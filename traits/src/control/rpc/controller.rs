//! Controller side for the [`Control`](super::Control) RPC set up.
//!
//! TODO!

// TODO: auto gen (proc macro, probably) the crimes below from the `Control`
// trait.

use super::{Encoding, EventFutureSharedState, Transport};
use super::{Control, ControlMessage};
use crate::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use crate::error::Error as Lc3Error;
use crate::memory::MemoryMiscError;
use crate::peripherals::adc::{AdcPinArr, AdcState, AdcReadError};
use crate::peripherals::gpio::{GpioPinArr, GpioState, GpioReadError};
use crate::peripherals::pwm::{PwmPinArr, PwmState};
use crate::peripherals::timers::{TimerArr, TimerState};
use lc3_isa::{Reg, Addr, Word};

use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};


// Converts calls on the control interface to messages and sends said messages.
#[derive(Debug, Default)]
pub struct Controller<'a, E, T, S>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    S: EventFutureSharedState,
{
    encoding: PhantomData<E>,
    pub transport: T,
    // pending_messages: Cell<[Option<ControlMessage>; 2]>,
    // pending_messages: [Option<ControlMessage>; 2],
    shared_state: &'a S,
    waiting_for_event: AtomicBool, // TODO: no reason for this to be Atomic // Note: it's atomic so we can maintain interior mutability?
    // waiting_for_event: bool,
}

impl<'a, E, T, S> Controller<'a, E, T, S>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    S: EventFutureSharedState,
{
    // When const functions can be in blanket impls, this can be made `const`.
    /*const*/ fn new(encoding: E, transport: T, shared_state: &'a S) -> Self {
        Self {
            encoding,
            transport,
            // pending_messages: Cell::new([None; 2]),
            // pending_messages: [None; 2],
            shared_state,
            waiting_for_event: AtomicBool::new(false),
            // waiting_for_event: false,
        }
    }
}

impl<'a, E, T, S> Controller<'a, E, T, S>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    S: EventFutureSharedStatePorcelain,
{
    // For now, we're going to assume sequential consistency (we receive
    // responses to messages in the same order we filed the requests). (TODO)
    //
    // Responses to our one non-blocking call (`run_until_event`) are the only
    // thing that could interrupt this.
    fn tick(&self) -> Option<ControlMessage> {
        let encoded_message = self.transport.get()?;
        let message = E::decode(&encoded_message).unwrap(); // TODO: don't panic;

        if let ControlMessage::RunUntilEventResponse(event) = message {
            if self.waiting_for_event.load(Ordering::SeqCst) {
                // println!("resolving the rpc future");
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
        use ControlMessage::*;
        $s.transport.send(E::encode($req).unwrap()).unwrap();

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
impl<'a, E, T, S> Control for Controller<'a, E, T, S>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    S: EventFutureSharedStatePorcelain,
{
    type EventFuture = EventFuture<'a, S>;

    fn get_pc(&self) -> Addr { ctrl!(self, GetPcRequest, GetPcResponse(addr), addr) }
    fn set_pc(&mut self, addr: Addr) { ctrl!(self, SetPcRequest { addr }, SetPcSuccess) }

    fn get_register(&self, reg: Reg) -> Word { ctrl!(self, GetRegisterRequest { reg }, GetRegisterResponse(word), word) }
    fn set_register(&mut self, reg: Reg, data: Word) { ctrl!(self, SetRegisterRequest { reg, data }, SetRegisterSuccess) }

    fn get_registers_psr_and_pc(&self) -> ([Word; Reg::NUM_REGS], Word, Word) {
        ctrl!(self, GetRegistersPsrAndPcRequest, GetRegistersPsrAndPcResponse(r), r)
    }

    fn read_word(&self, addr: Addr) -> Word { ctrl!(self, ReadWordRequest { addr }, ReadWordResponse(w), w) }
    fn write_word(&mut self, addr: Addr, word: Word) { ctrl!(self, WriteWordRequest { addr, word }, WriteWordSuccess) }
    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        ctrl!(self, CommitMemoryRequest, CommitMemoryResponse(r), r)
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        ctrl!(self, SetBreakpointRequest { addr }, SetBreakpointResponse(r), r)
    }
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        ctrl!(self, UnsetBreakpointRequest { idx }, UnsetBreakpointResponse(r), r)
    }
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] { ctrl!(self, GetBreakpointsRequest, GetBreakpointsResponse(r), r) }
    fn get_max_breakpoints(&self) -> usize { ctrl!(self, GetMaxBreakpointsRequest, GetMaxBreakpointsResponse(r), r) }

    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        ctrl!(self, SetMemoryWatchpointRequest { addr }, SetMemoryWatchpointResponse(r), r)
    }
    fn unset_memory_watchpoint(&mut self, idx: usize) -> Result<(), ()> {
        ctrl!(self, UnsetMemoryWatchpointRequest { idx }, UnsetBreakpointResponse(r), r)
    }
    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS] { ctrl!(self, GetMemoryWatchpointsRequest, GetMemoryWatchpointsResponse(r), r) }
    fn get_max_memory_watchpoints(&self) -> usize { ctrl!(self, GetMaxMemoryWatchpointsRequest, GetMaxMemoryWatchpointsResponse(r), r) }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {
        // If we're in a sealed batch with pending futures, just crash.
        self.shared_state.add_new_future().expect("no new futures once a batch starts to resolve");

        // If we're already waiting for an event, don't bother sending the
        // request along again:
        if !self.waiting_for_event.load(Ordering::SeqCst) {
            self.transport.send(E::encode(ControlMessage::RunUntilEventRequest).unwrap()).unwrap();
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
        // Self::tick(self).unwrap_none(); // when this goes stable, use this
        if Self::tick(self).is_some() { panic!("Controller received a message in tick!") }
    }

    fn step(&mut self) -> Option<Event> { ctrl!(self, StepRequest, StepResponse(r), r) }
    fn pause(&mut self) { ctrl!(self, PauseRequest, PauseSuccess) }

    fn get_state(&self) -> State { ctrl!(self, GetStateRequest, GetStateResponse(r), r) }

    fn reset(&mut self) {
        // Drop all pending futures:
        // (if one of these futures is polled again, bad bad things will happen; TODO)
        self.shared_state.reset();

        ctrl!(self, ResetRequest, ResetSuccess)
    }

    fn get_error(&self) -> Option<Lc3Error> { ctrl!(self, GetErrorRequest, GetErrorResponse(r), r) }

    // I/O Access:
    fn get_gpio_states(&self) -> GpioPinArr<GpioState> { ctrl!(self, GetGpioStatesRequest, GetGpioStatesResponse(r), r) }
    fn get_gpio_readings(&self) -> GpioPinArr<Result<bool, GpioReadError>> { ctrl!(self, GetGpioReadingsRequest, GetGpioReadingsResponse(r), r) }
    fn get_adc_states(&self) -> AdcPinArr<AdcState> { ctrl!(self, GetAdcStatesRequest, GetAdcStatesResponse(r), r) }
    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>> { ctrl!(self, GetAdcReadingsRequest, GetAdcReadingsResponse(r), r) }
    fn get_timer_states(&self) -> TimerArr<TimerState> { ctrl!(self, GetTimerStatesRequest, GetTimerStatesResponse(r), r) }
    fn get_timer_config(&self) -> TimerArr<Word> { ctrl!(self, GetTimerConfigRequest, GetTimerConfigResponse(r), r) }
    fn get_pwm_states(&self) -> PwmPinArr<PwmState> { ctrl!(self, GetPwmStatesRequest, GetPwmStatesResponse(r), r) }
    fn get_pwm_config(&self) -> PwmPinArr<u8> { ctrl!(self, GetPwmConfigRequest, GetPwmConfigResponse(r), r) }
    fn get_clock(&self) -> Word { ctrl!(self, GetClockRequest, GetClockResponse(r), r) }
}
