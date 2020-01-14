//! RPC for the Control trait.
//!
//! (TODO!)

use super::control::{Control, Event, State, MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use crate::error::Error as Lc3Error;
use crate::memory::MemoryMiscError;
use crate::peripherals::{adc::*, gpio::*, pwm::*, timers::*};
use lc3_isa::Reg;

use lc3_isa::*;

use core::task::Waker;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable};
use core::convert::Infallible;
use core::cell::Cell;
use core::num::NonZeroU8;
use core::sync::atomic::{AtomicBool, Ordering};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::fmt::Debug;

use serde::{Deserialize, Serialize};

static FOO: () = {
    let s = core::mem::size_of::<ControlMessage>();
    let canary = [()];

    canary[s - 64] // panic if the size of ControlMessage changes
};

// TODO: split into request/response types (helps with type safety (i.e. Device only
// deals with Responses) and potentially the size of the messages)
#[derive(Clone, Serialize, Deserialize, Debug)]
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

pub trait Encoding {
    type Encoded: Debug;
    type Err: Debug;

    fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err>;
    fn decode(encoded: &Self::Encoded) -> Result<ControlMessage, Self::Err>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TransparentEncoding;

impl Encoding for TransparentEncoding {
    type Encoded = ControlMessage;
    type Err = Infallible;

    fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err> {
        Ok(message)
    }

    fn decode(message: &Self::Encoded) -> Result<ControlMessage, Self::Err> {
        Ok(message.clone())
    }
}

pub trait Transport<EncodedFormat> {
    type Err: core::fmt::Debug;

    fn send(&self, message: EncodedFormat) -> Result<(), Self::Err>;

    // None if no messages were sent, Some(message) otherwise.
    fn get(&self) -> Option<EncodedFormat>; // TODO: should this be wrapped in a Result?
}

// TODO: for now, getting multiple futures at once (i.e. calling run_until_event
// twice) is somewhat undefined: at least one of the futures will eventually
// resolve but there's no guarantee on which one it will be. Additionally, all
// (or just more than one) of the futures may also resolve.

/// All these functions take an immutable reference to self so that instances of
/// the implementor can be shared between the future and the Controller
/// implementation.
///
/// This trait does not require that implementors be Sync (a requirement imposed
/// on Futures by certain executors) but some implementations will be (TODO).
///
/// Implementors are encouraged to provide a const-fn constructor so that
/// instances of the implementation can be put into `static` variables and
/// therefore be `'static' (this is desirable since most executors - at least
/// those that don't use scoped thread pools - require that the futures they
/// execute be static).
///
/// Here's how the flow is supposed to go.
///
/// First off we have to talk about what the discrete events are:
///   N: New future: the producer makes a new Future and gives a reference to an
///      instance of something that implements this trait. We'll call said
///      instance the state from here on out. The producer will call increment
///      on the state.
///   R: A future that the producer made is polled. The future calls
///      `get_event_and_state` to check if the producer has produced an event.
///      If it has, the future resolves and never calls anything on the state
///      again. The state decrements the count and if the count hits 0, drops
///      the event. We'll call this event R.
///   P: When the future goes to call `get_event_and_state` also possible is
///      that the producer hasn't made an event yet, in which case the future
///      simply registers its waker (again, potentially) and tries again later.
///      We'll call this P.
///   F: When the producer finally does make an event, it should inform the
///      state so that R can happen.
///
/// A couple more things:
///  - The first time `run_until_event` is called, a batch is created. Every
///    subsequent call to `run_until_event` *until an event actually happens*
///    produces a future that is part of this same batch.
///  - (NOTE: this is very much an arbitrary decision; in the future we might
///    decide that subsequent calls correspond to the next event, and the next
///    next event and so on. However such a system is likely to be more
///    confusing to users of the Control interface and will likely require a
///    heap backed queue or deque which is at odds with our goal of no_std
///    support. So, we'll stick with this system for now.)
///  - The key invariant we're trying to maintain is that batches *do not
///    overlap*. This means that once the producer makes an event, we cannot
///    make new futures (doing so starts a new batch) until all the existing
///    futures have resolved.
///  - (NOTE: this is also extremely arbitrary! Unlike the above, this isn't
///    very defensible; the primary reason we don't want batches to overlap
///    is simplicity. We could totally have an instance of the
///    `EventFutureSharedState` implementor per batch. This does, however, get
///    complicated quickly because, as mentioned, this state needs to be static
///    for many executors which makes having a variable number of them tricky.
///    However, having a fixed number of slots is entirely doable (not sure what
///    the failure mode would be though) and is something that can be done in
///    the future).
///
/// Now we can talk about ordering:
///  - We have to start with one or more Ns (since R, P, and F cannot happen
///    unless N has happened).
///  - From there futures can be polled (P) and new futures in the batch can be
///    created in any order. R depends on F and thus cannot happen yet.
///  - At some point, F will happen. Note that this can happen immediately after
///    the first N.
///  - Once F happens, the only thing that can follow F is X Rs where X is the
///    number of Ns between F and the last F. P cannot happen since any polls
///    will result in an R. We cannot allow new futures to be made (N) without
///    having overlapping batches. And F cannot happen again for the batch.
///
/// As a regex, kind of: `N(P*N*)*F(R){X}`
///
/// An example: `NPPPPPNPPPPPNNNPNPNPNPPNPPPPPPPPPPPPFRRRRRRRRR`.
///
/// The reset (starting state) should be: no waker, no event, count = 0.
///
/// Producers and Futures should probably use this trait's less prickly cousin,
/// [`EventFutureSharedStatePorcelain`], instead.
pub trait EventFutureSharedState {
    /// Implementors should hold onto at least the last waker passed in with
    /// this method.
    fn register_waker(&self, waker: Waker);

    /// Calls wake on the implementor's inner Waker(s) (if one or more are
    /// present).
    fn wake(&self);

    /// Should panic if called before all the futures depending on this instance
    /// call get_event_and_state.
    ///
    /// In normal use producers (i.e. not Futures) should call `is_clean` before
    /// calling this so that they don't panic.
    ///
    /// Implementors should not call `wake()` in this method. We'll let
    /// producers do that.
    fn set_event(&self, event: Event);

    /// Returns the state if it is present.
    ///
    /// Each time this is called while state *is* present, the count should be
    /// decremented. The state and any registered wakers disappear once the
    /// count hits 0.
    fn get_event(&self) -> Option<Event>;

    /// Increments the count of the number of futures that are out and using
    /// this instance to poll for the next event.
    ///
    /// Panics if we hit the maximum count.
    fn increment(&self) -> u8;

    /// `true` if `set_event` has been called on the state *and* there
    /// are still pending futures that need to be resolved.
    fn batch_sealed(&self) -> bool;

    /// `true` if the instance has no pending futures out waiting for it to;
    /// `false` otherwise.
    ///
    /// In other words, says whether or not the state is ready for a new batch.
    ///
    /// Note: this is no practical use for this function.
    fn is_clean(&self) -> bool;

    fn reset(&self); // TODO!
}


pub trait EventFutureSharedStatePorcelain: EventFutureSharedState {
    /// To be called by Futures.
    fn poll(&self, waker: Waker) -> Poll<Event> {
        if let Some(pair) = self.get_event() {
            Poll::Ready(pair)
        } else {
            self.register_waker(waker);
            Poll::Pending
        }
    }

    /// To be called by producers.
    fn add_new_future(&self) -> Result<&Self, ()> {
        if self.batch_sealed() {
            Err(())
        } else {
            self.increment();
            Ok(self)
        }
    }

    /// To be called by producers.
    fn resolve_all(&self, event: Event) -> Result<(), ()> {
        if self.batch_sealed() {
            Err(())
        } else {
            self.set_event(event);
            self.wake();

            Ok(())
        }
    }
}

// impl<E: EventFutureSharedState> EventFutureSharedStatePorcelain for E { }

#[derive(Debug, Clone)]
pub enum SharedStateState { // TODO: bad name, I know
    Errored,
    Dormant,
    WaitingForAnEvent { waker: Option<Waker>, count: NonZeroU8 },
    WaitingForFuturesToResolve { event: Event, waker: Option<Waker>, count: NonZeroU8 },
}

impl Default for SharedStateState {
    fn default() -> Self {
        Self::Errored
    }
}

impl SharedStateState {
    fn register_waker(&mut self, new_waker: Waker) {
        use SharedStateState::*;

        match self {
            Errored => unreachable!(),
            Dormant => panic!("Invariant violated: waker registered on shared state that has no attached futures."),
            WaitingForAnEvent { waker, count } => {
                if let Some(old) = waker {
                    if !new_waker.will_wake(&old) {
                        // This should *not* panic since this property isn't guaranteed
                        // even for the same future.
                        panic!("New waker doesn't wake the same futures as the old waker.")
                    }
                }

                *self = WaitingForAnEvent { waker: Some(new_waker), count: *count };
            },
            WaitingForFuturesToResolve { .. } => {
                panic!("Future registered a waker even though the event has happened!");
            }
        }
    }

    fn wake(&mut self) {
        use SharedStateState::*;

        match self {
            Errored | Dormant | WaitingForAnEvent { .. } => unreachable!(),
            WaitingForFuturesToResolve { waker, .. } => {
                // We'll only call the waker once!
                if let Some(waker) = waker.take() {
                    waker.wake();
                }
            }
        };
    }

    fn set_event(&mut self, event: Event) {
        use SharedStateState::*;

        match self {
            Errored | Dormant => panic!("Attempted to make an event without any futures!"),
            WaitingForFuturesToResolve { .. } => panic!("Attempted to make multiple events in a batch!"),
            WaitingForAnEvent { waker, count } => {
                *self = WaitingForFuturesToResolve { event, waker: waker.clone(), count: *count }
            }
        };
    }

    fn get_event(&mut self) -> Option<Event> {
        use SharedStateState::*;

        let ret = match self {
            Errored | Dormant => panic!("Unregistered future polled the state! {:?}", self),
            WaitingForFuturesToResolve { waker: Some(_), .. } => panic!("Waker persisted after batch was sealed!"),
            s @ WaitingForAnEvent { .. } => None,
            WaitingForFuturesToResolve { event, waker: None, count } => {
                if count.get() == 1 {
                    let event = *event;
                    *self = Dormant;
                    Some(event)
                } else {
                    *count = NonZeroU8::new(count.get() - 1).unwrap();
                    Some(*event)
                }
            },
        };

        ret
    }

    fn increment(&mut self) -> u8 {
        use SharedStateState::*;

        let ret = match self {
            Errored => unreachable!(),
            WaitingForFuturesToResolve { .. } => panic!("Attempted to add a future to a sealed batch!"),
            Dormant => {
                *self = WaitingForAnEvent { waker: None, count: NonZeroU8::new(1).unwrap() };
                1
            },
            WaitingForAnEvent { count, .. } => {
                *count = NonZeroU8::new(count.get().checked_add(1).unwrap()).unwrap();
                count.get()
            }
        };

        ret
    }

    fn batch_sealed(&self) -> bool {
        if let SharedStateState::WaitingForFuturesToResolve { .. } = self {
            true
        } else {
            false
        }
    }

    fn is_clean(&self) -> bool {
        if let SharedStateState::Dormant = self {
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        *self =  SharedStateState::Dormant; // TODO: currently pending futures!
    }
}

pub struct SimpleEventFutureSharedState {
    // waker: Cell<Option<Waker>>,
    // count: AtomicU8,
    // state: Cell<Option<(Event, State)>>,
    inner: Cell<SharedStateState>
}

impl SimpleEventFutureSharedState {
    pub const fn new() -> Self {
        Self {
            // waker: Cell::new(None),
            // count: AtomicU8::new(0),
            // state: Cell::new(None),
            inner: Cell::new(SharedStateState::Dormant),
        }
    }

    fn update<R>(&self, func: impl FnOnce(&mut SharedStateState) -> R) -> R {
        let mut s = self.inner.take();

        let res = func(&mut s);
        self.inner.set(s);

        res
    }
}

impl EventFutureSharedStatePorcelain for SimpleEventFutureSharedState { }

impl EventFutureSharedState for SimpleEventFutureSharedState {

    fn register_waker(&self, new_waker: Waker) {
        self.update(|s| s.register_waker(new_waker))
    }

    fn wake(&self) {
        self.update(|s| s.wake())
    }

    fn set_event(&self, event: Event) {
        self.update(|s| s.set_event(event))
    }

    fn get_event(&self) -> Option<Event> {
        self.update(|s| s.get_event())
    }

    fn increment(&self) -> u8 {
        self.update(|s| s.increment())
    }

    fn batch_sealed(&self) -> bool {
        self.update(|s| s.batch_sealed())
    }

    fn is_clean(&self) -> bool {
        self.update(|s| s.is_clean())
    }

    fn reset(&self) {
        self.update(|s| s.reset())
    }
}

#[derive(Debug)]
pub struct EventFuture<'a, S: EventFutureSharedState>(&'a S);

impl<'a, S: EventFutureSharedStatePorcelain> EventFuture<'a, S> {
    pub /*const*/ fn new(inner: &'a S) -> Self {
        Self(inner)
    }
}

impl<'a, S: EventFutureSharedStatePorcelain> Future for EventFuture<'a, S> {
    type Output = Event;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll(ctx.waker().clone())
    }
}

// TODO: Add tokio tracing to this (behind a feature flag) or just the normal
// log crate.

// TODO: auto gen (proc macro, probably) the nice type up above from and the
// crimes below.

#[derive(Debug)]
pub struct Controller<'a, E, T, S>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    S: EventFutureSharedState,
{
    pub encoding: E,
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
        let message = E::decode(&encoded_message).unwrap();

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

// Check for messages and execute them on something that implements the control
// interface.
#[derive(Debug, Default)]
pub struct Device<E, T, C>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    C: Control,
    // <C as Control>::EventFuture: Unpin,
{
    pub encoding: E,
    pub transport: T,
    _c: PhantomData<C>,
    // pending_event_future: Option<Pin<C::EventFuture>>,
    pending_event_future: Option<C::EventFuture>,
}

impl<E, T, C> Device<E, T, C>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    C: Control,
    // <C as Control>::EventFuture: Unpin,
{
    // When const functions can be in blanket impls, this can be made `const`.
    /*const*/ fn new(encoding: E, transport: T) -> Self {
        Self {
            encoding,
            transport,
            _c: PhantomData,
            pending_event_future: None,
        }
    }
}

// This is meant to be a barebones blocking executor.
// It has been updated to more or less mirror the 'executor' in
// [`genawaiter`](https://docs.rs/crate/genawaiter/0.2.2/source/src/waker.rs);
// while this doesn't guarentee correctness, it does make me feel a little
// better.
//
// Another option would have been to use (or steal) `block_on` from the
// `futures` crate. We may switch to doing this in the future but currently we
// don't because:
//   - really, we want to _poll_ the future in the step function and not block
//     on it
//   - because the future that the simulator (i.e. the Control implementor)
//     that's given to a Device instance is ostensibly an EventFuture, it's
//     not going to need to do any real I/O; we can absolutely get away with
//     a fake executor that doesn't have a reactor and never actually does
//     any scheduling
//
// This is fine for now, but is definitely not ideal. If, in the future, we
// write simulators that do real async I/O and produce real Futures, we may need
// to use an executor from `tokio` (a cursory glance through the executors in
// `futures` seems to suggest they don't do anything special to accommodate I/O
// but I'm probably wrong). IIUC, this is mostly a performance thing; without an
// actual scheduler we'll just be blindly calling poll far more often than we
// actually should.
//
// I'm okay with this for now, since:
//   - it's not super apparent to me what an actual async API for the simulator
//     (i.e. an async version of Control) would look like
//   - we can't do async in Traits yet (_properly_) anyways
//
// (Glancing through `async-std` seems to confirm that the executor, the reactor
// and the futures are somewhat decoupled: the executor bumbles along trying to
// finish tasks and very politely asking futures to let it know when tasks that
// are blocked will become unblocked (i.e. the Waker API); futures are to call
// out to the actual hardware to do work and also to _arrange_ for the executor
// to be notified when they can make further process; the reactor is the thing
// that arranges for whatever is underneath us to alert the executor when a
// certain task can make progress. This suggests that individual futures can be
// somewhat tied to reactor implementations. For example, `async-std` uses `mio`
// and would not function correctly if the futures its functions produces were
// executed without a Mio based reactor being present; instead the futures would
// report that they were NotReady, would try to register themselves with their
// Mio based reactor (passing along their context or just their Waker) and would
// then be queued by their executor, unaware that no arrangement had actually
// been made to inform the executor once they could be awoken. All this is to
// say that futures and their reactor definitely are coupled and it seems like
// an executor must either start a reactor thread (that futures then somehow
// communicate with??) or there must be some way to guarantee that a globally
// accessible reactor instance will be available and running. Again, I've only
// glanced through async-std but it seems to opt for something like the latter;
// Reactor instances (per thing, I think -- like net has it's own reactor) are
// global and -- cleverly -- only started once the global variable the instance
// lives in is accessed. This lets the executor be largely unaware of the
// reactor, I think. Still not totally clear on how the futures interact with
// Mio (the reactor) but it has to do with `Evented` and the types Mio exposes.
// I have basically not looked at `tokio` at all but I think it does something
// fancier with layers (the executor, some timers, and the reactor are layers,
// in that order, I think)).
//
// [Reactor]: {Registers events w/kernel}  <-----
//    (proxies) | (wake these)                   \
//              | ( futures  )                   |
//              |               ----> {Register Wake} -> [Sleep]
//              v              /    ^----------------|
// future -> [Executor] -> Task-----> [Running] -----|
//                             \    ^-v--------------|
//                              ------> [Finished]
//
//
// Sidenote: the desire for these pieces to be decoupled is why the Waker API
// is as "build it yourself" as it is iiuc (i.e. the raw vtable); they couldn't
// do a trait because then everything would have to be generic over the Waker;
// they couldn't do a trait object because then object safety rears its head
// (associated types? i think) so: all type safety was sacrificed. iirc one of
// withoutboats' async blog posts talks about it.
//
// Anyways. Now that we've got something of an understanding, we can talk about
// our fairly simple use case.
//
// For clarity, here's our whole picture:
//
// ```
//   /----------------------------------------------------------------------\
//  |                    [Controller Side: i.e. Laptop]                      |
//  |                                                                        |
//  |  /----------------------\                     %%% < %%%                |
//  | | [Controller]: `Control`|               %%% [Main Loop] %%%           |
//  | | tick:                  |                                             |
//  | |  - resolves futures    |           %%%  /---------------\  %%%       |
//  | |    issued by           |               |  [Client Logic] |           |
//  | |    `run_until_event`   |<---\     %%%  |                 |   %%%     |
//  | | rest:                  |    |     vvv  | Uses the device |   ^^^     |
//  | |  - proxied; send req.  |    |     %%%  | via the Control |   %%%     |
//  | |    and block on resp.  |    |          | interface.      |           |
//  |  \--|----------------^--/     |     %%%  |  /---^          |  %%%      |
//  |     |                |        |           \-|-------------/            |
//  | |---v----|     |-----|---|    |        %%%  v              %%%         |
//  | |Enc: Req|     |Dec: Resp|    \----------->[Control::tick]             |
//  | |-|------|     |-------^-|                    %%% > %%%                |
//   \--|--------------------|----------------------------------------------/
//      |<Con Send  Con Recv>|
//      |  [Transport Layer] |
//      |<Dev Recv  Dev Send>|
//   /--v--------------------|----------------------------------------------\
//  | |--------|     |-------|-|            %%% < %%%            /--------\  |
//  | |Dec: Req|     |Enc: Resp|       %%% [Dev. Loop] %%%      |  [Sim.]  | |
//  | |---|----|     |-----^---|                       /--------| ╭──────╮ | |
//  |     |                |       %%%                 |   %%%  | │Interp│ | |
//  |  /--v----------------|--\                        |        | ╰──────╯ | |
//  | |        [Device]        |  %%%                  v     %%% \--------/  |
//  | | tick:                  |  vvv [Device::tick(......)] ^^^             |
//  | |  - makes progress on   |  %%%     |                  %%%             |
//  | |    any futures that    |<---------/                                  |
//  | |    were issued         |  %%%                       %%%              |
//  | |  - processes new reqs  |                                             |
//  | |    (blocks if not a    |     %%%  v              %%%                 |
//  | |    `run_until_event`)  |                                             |
//  |  \----------------------/             %%% > %%%                        |
//  |                                                                        |
//  |                         [Device Side: i.e. TM4C]                       |
//   \----------------------------------------------------------------------/
// ```

// ╭──────╮
// │Interp│
// ╰──────╯
// ╔──────╗
// │Interp│
// ╚──────╝
// ╔══════╗
// ║Interp║
// ╚══════╝

static NO_OP_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    RW_CLONE,
    RW_WAKE,
    RW_WAKE_BY_REF,
    RW_DROP
);

#[doc(hidden)]
pub static RW_CLONE: fn(*const ()) -> RawWaker = |_| RawWaker::new(
    core::ptr::null(),
    &NO_OP_RAW_WAKER_VTABLE,
);
static RW_WAKE: fn(*const ()) = |_| { };
static RW_WAKE_BY_REF: fn(*const ()) = |_| { };
static RW_DROP: fn(*const ()) = |_| { };

impl<E, T, C> Device<E, T, C>
where
    E: Encoding,
    T: Transport<<E as Encoding>::Encoded>,
    C: Control,
    <C as Control>::EventFuture: Unpin, // TODO: use `pin_utils::pin_mut!` and relax this requirement.
    // <C as Control>::EventFuture: Deref<Target = <C as Control>::EventFuture>,
    // <C as Control>::EventFuture: Deref,
    // <C as Control>::EventFuture: DerefMut,
    // <<C as Control>::EventFuture as Deref>::Target: Future<Output = (Event, State)>,
    // <<C as Control>::EventFuture as Deref>::Target: Unpin,
{
    #[allow(unsafe_code)]
    pub fn step(&mut self, c: &mut C) -> usize {
        use ControlMessage::*;
        let mut num_processed_messages = 0;

        // Make some progress:
        c.tick();

        if let Some(ref mut f) = self.pending_event_future {
            // println!("polling the device future");

            // TODO: we opt to poll here because we assume that the underlying future is
            // rubbish so our waker (if we were to register a real one) would never be called.
            //
            // However, the simulator's future (just as the one the controller exposes) does
            // treat the waker correctly. Additionally if someone were to write a truly
            // async simulator, this would also be a real future that respects the waker.
            //
            // So, it may be worth looking into using a real waker that notifies us that
            // something has happened. Or better yet, maybe writing an async Device rpc
            // thing that just chains our future onto the real one.
            //
            // On the other hand, this is at odds with no_std support and it's unlikely
            // to net material performance wins so, maybe eventually.
            if let Poll::Ready(event) = Pin::new(f).poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RW_CLONE(&())) } )) {
                // println!("device future is done!");
                self.pending_event_future = None;

                let enc = E::encode(RunUntilEventResponse(event)).unwrap();
                self.transport.send(enc).unwrap();
            }
        }

        while let Some(m) = self.transport.get().map(|enc| E::decode(&enc).unwrap()) {
            num_processed_messages += 1;

            macro_rules! dev {
                ($(($req:pat => $($resp:tt)+) with $r:tt = $resp_expr:expr;)*) => {
                    #[forbid(unreachable_patterns)]
                    match m {
                        RunUntilEventRequest => {
                            if self.pending_event_future.is_some() {
                                panic!() // TODO: write a message // already have a run until event pending!
                            } else {
                                // self.pending_event_future = Some(Pin::new(c.run_until_event()));
                                self.pending_event_future = Some(c.run_until_event());
                            }
                        },
                        RunUntilEventResponse(_) => panic!("Received a run_until_event response on the device side!"),
                        $(
                            $req => self.transport.send(E::encode({
                                let $r = $resp_expr;
                                $($resp)+
                            }).unwrap()).unwrap(),
                            #[allow(unused_variables)]
                            $($resp)+ => panic!("Received a response on the device side!"),
                        )*
                    }

                };
            }

            dev!{
                (GetPcRequest => GetPcResponse(r)) with r = c.get_pc();
                (SetPcRequest { addr } => SetPcSuccess) with _ = c.set_pc(addr);

                (GetRegisterRequest { reg } => GetRegisterResponse(r)) with r = c.get_register(reg);
                (SetRegisterRequest { reg, data } => SetRegisterSuccess) with _ = c.set_register(reg, data);

                (GetRegistersPsrAndPcRequest => GetRegistersPsrAndPcResponse(r)) with r = c.get_registers_psr_and_pc();

                (ReadWordRequest { addr } => ReadWordResponse(r)) with r = c.read_word(addr);
                (WriteWordRequest { addr, word } => WriteWordSuccess) with _ = c.write_word(addr, word);

                (CommitMemoryRequest => CommitMemoryResponse(r)) with r = c.commit_memory();

                (SetBreakpointRequest { addr } => SetBreakpointResponse(r)) with r= c.set_breakpoint(addr);
                (UnsetBreakpointRequest { idx } => UnsetBreakpointResponse(r)) with r = c.unset_breakpoint(idx);
                (GetBreakpointsRequest => GetBreakpointsResponse(r)) with r = c.get_breakpoints();
                (GetMaxBreakpointsRequest => GetMaxBreakpointsResponse(r)) with r = c.get_max_breakpoints();

                (SetMemoryWatchpointRequest { addr } => SetMemoryWatchpointResponse(r)) with r = c.set_memory_watchpoint(addr);
                (UnsetMemoryWatchpointRequest { idx } => UnsetMemoryWatchpointResponse(r)) with r = c.unset_memory_watchpoint(idx);
                (GetMemoryWatchpointsRequest => GetMemoryWatchpointsResponse(r)) with r = c.get_memory_watchpoints();
                (GetMaxMemoryWatchpointsRequest => GetMaxMemoryWatchpointsResponse(r)) with r = c.get_max_memory_watchpoints();

                (StepRequest => StepResponse(r)) with r = c.step();
                (PauseRequest => PauseSuccess) with _ = c.pause();
                (GetStateRequest => GetStateResponse(r)) with r = c.get_state();
                (ResetRequest => ResetSuccess) with _ = c.reset();

                (GetErrorRequest => GetErrorResponse(r)) with r = c.get_error();

                (GetGpioStatesRequest => GetGpioStatesResponse(r)) with r = c.get_gpio_states();
                (GetGpioReadingsRequest => GetGpioReadingsResponse(r)) with r = c.get_gpio_readings();

                (GetAdcStatesRequest => GetAdcStatesResponse(r)) with r = c.get_adc_states();
                (GetAdcReadingsRequest => GetAdcReadingsResponse(r)) with r = c.get_adc_readings();

                (GetTimerStatesRequest => GetTimerStatesResponse(r)) with r = c.get_timer_states();
                (GetTimerConfigRequest => GetTimerConfigResponse(r)) with r = c.get_timer_config();

                (GetPwmStatesRequest => GetPwmStatesResponse(r)) with r = c.get_pwm_states();
                (GetPwmConfigRequest => GetPwmConfigResponse(r)) with r = c.get_pwm_config();

                (GetClockRequest => GetClockResponse(r)) with r = c.get_clock();
            };
        }

        num_processed_messages
    }
}

using_std! {
    use std::sync::RwLock;
    use std::sync::mpsc::{Sender, Receiver, SendError};

    #[derive(Debug)]
    pub struct SyncEventFutureSharedState(RwLock<SharedStateState>);

    impl SyncEventFutureSharedState {
        // This, unfortunately, can't be const. Users will need to use
        // lazy_static or something similar.
        pub fn new() -> Self {
            Self(RwLock::new(SharedStateState::Dormant))
        }
    }

    // It's not great that we have to do this... maybe we shouldn't have the default impl (TODO) or at least the blanket impl (done)
    impl EventFutureSharedStatePorcelain for SyncEventFutureSharedState {
        fn poll(&self, waker: Waker) -> Poll<Event> {
            let mut s = self.0.write().unwrap();

            if let Some(pair) = s.get_event() {
                Poll::Ready(pair)
            } else {
                s.register_waker(waker);
                Poll::Pending
            }
        }

        fn add_new_future(&self) -> Result<&Self, ()> {
            let mut s = self.0.write().unwrap();

            if s.batch_sealed() {
                Err(())
            } else {
                s.increment();
                Ok(self)
            }
        }

        fn resolve_all(&self, event: Event) -> Result<(), ()> {
            let mut s = self.0.write().unwrap();

            if s.batch_sealed() {
                Err(())
            } else {
                s.set_event(event);
                s.wake();

                Ok(())
            }
        }
    }

    impl EventFutureSharedState for SyncEventFutureSharedState {
        fn register_waker(&self, waker: Waker) {
            self.0.write().unwrap().register_waker(waker)
        }

        fn wake(&self) {
            self.0.write().unwrap().wake()
        }

        fn set_event(&self, event: Event) {
            self.0.write().unwrap().set_event(event)
        }

        fn get_event(&self) -> Option<Event> {
            self.0.write().unwrap().get_event()
        }

        fn increment(&self) -> u8 {
            self.0.write().unwrap().increment()
        }

        fn batch_sealed(&self) -> bool {
            self.0.read().unwrap().batch_sealed()
        }

        fn is_clean(&self) -> bool {
            self.0.read().unwrap().is_clean()
        }

        fn reset(&self) {
            self.0.write().unwrap().reset();
        }
    }

    #[cfg(feature = "json_encoding")]
    pub struct JsonEncoding;

    #[cfg(feature = "json_encoding")]
    impl Encoding for JsonEncoding {
        type Encoded = String;
        type Err = serde_json::error::Error;

        fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err> {
            serde_json::to_string(&message)
        }

        fn decode(encoded: &Self::Encoded) -> Result<ControlMessage, Self::Err> {
            serde_json::from_str(encoded)
        }
    }

    pub struct MpscTransport<EncodedFormat: Debug> {
        tx: Sender<EncodedFormat>,
        rx: Receiver<EncodedFormat>,
    }

    impl<EncodedFormat: Debug> Transport<EncodedFormat> for MpscTransport<EncodedFormat> {
        type Err = SendError<EncodedFormat>;

        fn send(&self, message: EncodedFormat) -> Result<(), Self::Err> {
            log::trace!("SENT: {:?}", message);
            self.tx.send(message)
        }

        fn get(&self) -> Option<EncodedFormat> {
            if let Ok(m) = self.rx.try_recv() {
                log::trace!("GOT: {:?}", m);
                Some(m)
            } else {
                None
            }

            // TODO(fix): this breaks `run_until_event`!!
            // Going to use this blocking variant for now even though it is likely to
            // result in worse performance for huge amounts of messages
            // let m = self.rx.recv().ok();
            // log::trace!("GOT: {:?}", m);
            // m
        }
    }

    impl<EncodedFormat: Debug> MpscTransport<EncodedFormat> {
        pub fn new() -> (Self, Self) {
            mpsc_transport_pair()
        }
    }

    fn mpsc_transport_pair<C: Debug>() -> (MpscTransport<C>, MpscTransport<C>) {
        let (tx_h, rx_h) = std::sync::mpsc::channel();
        let (tx_d, rx_d) = std::sync::mpsc::channel();

        let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
        let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

        (host_channel, device_channel)
    }

    pub fn mpsc_sync_pair<'a, Enc: Encoding + Default/* = TransparentEncoding*/, C: Control>(state: &'a SyncEventFutureSharedState) -> (Controller<'a, Enc, MpscTransport<Enc::Encoded>, SyncEventFutureSharedState>, Device<Enc, MpscTransport<Enc::Encoded>, C>)
    {
        let (controller, device) = MpscTransport::new();

        let controller = Controller::new(Enc::default(), controller, state);
        let device = Device::new(Enc::default(), device);

        (controller, device)
    }
}
