// extern crate futures; // 0.1.23
// extern crate rand;

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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ControlMessage {
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
    GetRegistersPsrAndPcResponse([Word; Reg::NUM_REGS], Word, Word),

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
    RunUntilEventResponse(Event, State),
    // TODO: add a quick immediate response message (probably should do this!)
    // (call it success!)

    StepRequest,
    StepResponse(State),

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
    type Encoded;
    type Err: core::fmt::Debug;

    fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err>;
    fn decode(encoded: &Self::Encoded) -> Result<ControlMessage, Self::Err>;
}

pub struct TransparentEncoding;

impl Encoding for TransparentEncoding {
    type Encoded = ControlMessage;
    type Err = Infallible;

    fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err> {
        Ok(message)
    }

    fn decode(message: &Self::Encoded) -> Result<ControlMessage, Self::Err> {
        Ok(*message)
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
    fn set_event_and_state(&self, event: Event, state: State);

    /// Returns the state if it is present.
    ///
    /// Each time this is called while state *is* present, the count should be
    /// decremented. The state and any registered wakers disappear once the
    /// count hits 0.
    fn get_event_and_state(&self) -> Option<(Event, State)>;

    /// Increments the count of the number of futures that are out and using
    /// this instance to poll for the next event.
    ///
    /// Panics if we hit the maximum count.
    fn increment(&self) -> u8;

    /// `true` if `set_event_and_state` has been called on the state *and* there
    /// are still pending futures that need to be resolved.
    fn batch_sealed(&self) -> bool;

    /// `true` if the instance has no pending futures out waiting for it to;
    /// `false` otherwise.
    ///
    /// In other words, says whether or not the state is ready for a new batch.
    ///
    /// Note: this is no practical use for this function.
    fn is_clean(&self) -> bool;
}


pub trait EventFutureSharedStatePorcelain: EventFutureSharedState {
    /// To be called by Futures.
    fn poll(&self, waker: Waker) -> Poll<(Event, State)> {
        if let Some(pair) = self.get_event_and_state() {
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
    fn resolve_all(&self, event: Event, state: State) -> Result<(), ()> {
        if self.batch_sealed() {
            Err(())
        } else {
            self.set_event_and_state(event, state);
            self.wake();

            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum SharedStateState { // TODO: bad name, I know
    Errored,
    Dormant,
    WaitingForAnEvent { waker: Option<Waker>, count: NonZeroU8 },
    WaitingForFuturesToResolve { pair: (Event, State), waker: Option<Waker>, count: NonZeroU8 },
}

impl Default for SharedStateState {
    fn default() -> Self {
        Self::Errored
    }
}

pub struct SimpleEventFutureSharedState {
    // waker: Cell<Option<Waker>>,
    // count: AtomicU8,
    // state: Cell<Option<(Event, State)>>,
    inner: Cell<SharedStateState>
}

impl SimpleEventFutureSharedState {
    const fn new() -> Self {
        Self {
            // waker: Cell::new(None),
            // count: AtomicU8::new(0),
            // state: Cell::new(None),
            inner: Cell::new(SharedStateState::Dormant),
        }
    }
}

impl EventFutureSharedState for SimpleEventFutureSharedState {

    fn register_waker(&self, new_waker: Waker) {
        use SharedStateState::*;
        let s = self.inner.take();

        match s {
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

                self.inner.set(WaitingForAnEvent { waker: Some(new_waker), count } )
            },
            WaitingForFuturesToResolve { .. } => {
                panic!("Future registered a waker even though the event has happened!");
            }
        }

        // if let Some(old) = self.waker.get() {
        //     if self.is_clean() {
        //         panic!("Invariant violated: waker registered on shared state that has no attached futures.")
        //     }

        //     if !waker.will_wake(old) {
        //         // This should *not* panic since this property isn't guaranteed
        //         // even for the same future.
        //         panic!("New waker doesn't wake the same futures as the old waker.")
        //     }
        // }

        // self.waker.set(Some(waker))
    }

    fn wake(&self) {
        use SharedStateState::*;
        let s = self.inner.take();

        self.inner.set(match s {
            Errored | Dormant | WaitingForAnEvent { .. } => unreachable!(),
            WaitingForFuturesToResolve { pair, waker, count } => {
                if let Some(waker) = waker {
                    waker.wake()
                }

                // We'll only call the waker once!
                WaitingForFuturesToResolve { pair, waker: None, count }
            }
        });
    }

    fn set_event_and_state(&self, event: Event, state: State) {
        use SharedStateState::*;
        let s = self.inner.take();

        self.inner.set(match s {
            Errored | Dormant => panic!("Attempted to make an event without any futures!"),
            WaitingForFuturesToResolve { .. } => panic!("Attempted to make multiple events in a batch!"),
            WaitingForAnEvent { waker, count } => {
                WaitingForFuturesToResolve { pair: (event, state), waker, count }
            }
        });
    }

    fn get_event_and_state(&self) -> Option<(Event, State)> {
        use SharedStateState::*;
        let s = self.inner.take();

        let (next, ret) = match s {
            Errored | Dormant => panic!("Unregistered future polled the state!"),
            WaitingForFuturesToResolve { waker: Some(_), .. } => panic!("Waker persisted after batch was sealed!"),
            s @ WaitingForAnEvent { .. } => (s, None),
            WaitingForFuturesToResolve { pair, waker: None, count } => {
                if count.get() == 1 {
                    (Dormant, Some(pair))
                } else {
                    (
                        WaitingForFuturesToResolve {
                            pair,
                            waker: None,
                            count: NonZeroU8::new(count.get() - 1).unwrap(),
                        },
                        Some(pair)
                    )
                }
            },
        };

        self.inner.set(next);
        ret
    }

    fn increment(&self) -> u8 {
        use SharedStateState::*;
        let s = self.inner.take();

        let (next, ret) = match s {
            Errored => unreachable!(),
            WaitingForFuturesToResolve { .. } => panic!("Attempted to add a future to a sealed batch!"),
            Dormant => {
                (WaitingForAnEvent { waker: None, count: NonZeroU8::new(1).unwrap() }, 1)
            },
            WaitingForAnEvent { waker, count } => {
                let count = count.get().checked_add(1).unwrap();
                (WaitingForAnEvent { waker, count: NonZeroU8::new(count).unwrap() }, count)
            }
        };

        self.inner.set(next);
        ret
    }

    fn batch_sealed(&self) -> bool {
        let s = self.inner.take();

        let res = if let SharedStateState::WaitingForFuturesToResolve { .. } = s {
            true
        } else {
            false
        };

        self.inner.set(s);
        res
    }

    fn is_clean(&self) -> bool {
        let s = self.inner.take();

        let res = if let SharedStateState::Dormant = s {
            true
        } else {
            false
        };

        self.inner.set(s);
        res
    }
}

pub struct EventFuture<'a, S: EventFutureSharedState>(&'a S);

impl<'a, S: EventFutureSharedStatePorcelain> Future for EventFuture<'a, S> {
    type Output = (Event, State);

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll(ctx.waker().clone())
    }
}

        if let Some(m) = self.transport.get() {
            if let Message::GET_PWM_STATES_RETURN_VAL(states) = m {
                ret = states;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }
    fn get_pwm_config(&self) -> PwmPinArr<u8> {
        let mut ret: PwmPinArr<u8>;
        self.transport.send(Message::GET_PWM_CONFIG);

        if let Some(m) = self.transport.get() {
            if let Message::GET_PWM_CONFIG_RETURN_VAL(conf) = m {
                ret = conf;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }
    fn get_clock(&self) -> Word {
        16
    }
    fn reset(&mut self) {}
}

// TODO: rename to Device? Or Source?
/// Check for messages and execute them on something that implements the control
/// interface.
pub struct Client<T: TransportLayer> {
    pub transport: T,
}

impl<T: TransportLayer> Client<T> {
    pub fn step<C: Control>(&mut self, cont: &mut C) -> usize {
        let mut num_executed_messages = 0;

        while let Some(m) = self.transport.get() {
            use Message::*;

            num_executed_messages += 1;

            match m {
                GET_PC => {
                    self.transport.send(GET_PC_RETURN_VAL(cont.get_pc()));
                }

                GET_PC_RETURN_VAL(h) => {}

                SET_PC(val) => {
                    cont.set_pc(val);
                    self.transport.send(SET_PC_SUCCESS);
                }

                SET_REGISTER(reg, word) => {
                    cont.set_register(reg, word);
                    self.transport.send(SET_REGISTER_SUCCESS);
                }

                RUN_UNTIL_EVENT => {
                    cont.run_until_event();
                    self.transport.send(ISSUED_RUN_UNTIL_EVENT);
                }

                WRITE_WORD(word, value) => {
                    cont.write_word(word, value);
                    self.transport.send(WRITE_WORD_SUCCESS);
                }

                PAUSE => {
                    cont.pause();
                    self.transport.send(PAUSE_SUCCESS);
                }

                SET_BREAKPOINT(addr) => {
                    cont.set_breakpoint(addr);
                    self.transport.send(SET_BREAKPOINT_SUCCESS);
                }

                UNSET_BREAKPOINT(addr) => {
                    cont.unset_breakpoint(addr);
                    self.transport.send(UNSET_BREAKPOINT_SUCCESS);
                }

                GET_BREAKPOINTS => {
                    let breaks = cont.get_breakpoints();
                    self.transport.send(GET_BREAKPOINTS_RETURN_VAL(breaks));
                }

                SET_MEMORY_WATCH(addr, word) => {
                    let res = cont.set_memory_watch(addr, word);
                    self.transport.send(SET_MEMORY_WATCH_SUCCESS(res));
                }

                UNSET_MEMORY_WATCH(idx) => {
                    let res = cont.unset_memory_watch(idx);
                    self.transport.send(UNSET_MEMORY_WATCH_SUCCESS(res));
                }

                GET_MAX_BREAKPOINTS => (),    // TODO: do

                GET_MAX_MEMORY_WATCHES => (), // TODO: do

                STEP => {
                    let state = cont.step();
                    self.transport.send(STEP_RETURN_STATE(state));
                }

                READ_WORD(addr) => {
                    self.transport
                        .send(READ_WORD_RETURN_VAL(cont.read_word(addr)));
                }
                GET_MEMORY_WATCHES => {
                    cont.get_memory_watches();
                }

                GET_REGISTER(reg) => {
                    self.transport
                        .send(GET_REGISTER_RETURN_VAL(cont.get_register(reg)));
                }

                COMMIT_MEMORY => {
                    let res = cont.commit_memory();
                    self.transport.send(COMMIT_MEMORY_SUCCESS(res));
                }

                GET_STATE => {
                    let state = cont.get_state();
                    self.transport.send(GET_STATE_RETURN_VAL(state));
                }

                GET_GPIO_STATES => {
                    let state = cont.get_gpio_states();
                    self.transport.send(GET_GPIO_STATES_RETURN_VAL(state));
                }

                GET_GPIO_READING => {
                    let state = cont.get_gpio_reading();
                    self.transport.send(GET_GPIO_READING_RETURN_VAL(state));
                }

                GET_ADC_READING => {
                    let state = cont.get_adc_reading();
                    self.transport.send(GET_ADC_READING_RETURN_VAL(state));
                }

                GET_ADC_STATES => {
                    let state = cont.get_adc_states();
                    self.transport.send(GET_ADC_STATES_RETURN_VAL(state));
                }

                GET_TIMER_STATES => {
                    let state = cont.get_timer_states();
                    self.transport.send(GET_TIMER_STATES_RETURN_VAL(state));
                }

                GET_TIMER_CONFIG => {
                    let state = cont.get_timer_config();
                    self.transport.send(GET_TIMER_CONFIG_RETURN_VAL(state));
                }

                GET_PWM_STATES => {
                    let state = cont.get_pwm_states();
                    self.transport.send(GET_PWM_STATES_RETURN_VAL(state));
                }

                GET_PWM_CONFIG => {
                    let state = cont.get_pwm_config();
                    self.transport.send(GET_PWM_CONFIG_RETURN_VAL(state));
                }

                GET_CLOCK => {
                    let state = cont.get_clock();
                    self.transport.send(GET_CLOCK_RETURN_VAL(state));
                }

                _ => unreachable!(),
            }
        }

        num_executed_messages
    }
}

using_std! {
    use std::sync::RwLock;
    use std::sync::mpsc::{Sender, Receiver, SendError};

    // TODO: this is a little silly since we've essentially got
    // `RwLock<Cell<SharedStateState>>`. We can ditch the Cell since the RwLock
    // affords us interior mutability.
    //
    // Didn't want to duplicate all the logic in the trait impl for
    // `SimpleEventFutureSharedState` yet though, which is why this even exists.
    struct SyncEventFutureSharedState(RwLock<SimpleEventFutureSharedState>);

    impl SyncEventFutureSharedState {
        // This, unfortunately, can't be const. Users will need to use
        // lazy_static or something similar.
        fn new() -> Self {
            Self(RwLock::new(SimpleEventFutureSharedState::new()))
        }
    }

    impl EventFutureSharedState for SyncEventFutureSharedState {
        fn register_waker(&self, waker: Waker) {
            self.write().register_waker(waker)
        }

        fn wake(&self) {
            self.write().wake()
        }

        fn set_event_and_state(&self, event: Event, state: State) {
            self.write().set_event_and_state(event, state)
        }

        fn get_event_and_state(&self) -> Option<(Event, State)> {
            self.write().get_event_and_state()
        }

        fn increment(&self) -> u8 {
            self.write().increment()
        }

        fn batch_sealed(&self) -> bool {
            self.read().batch_sealed()
        }

        fn is_clean(&self) -> bool {
            self.read().is_clean()
        }
    }

//     let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
//     let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

//     (host_channel, device_channel)
// }

// //fn run_channel()

using_std! {
    use std::sync::mpsc::{Sender, Receiver};

    pub struct MpscTransport {
        tx: Sender<std::string::String>,
        rx: Receiver<std::string::String>,
    }

    impl TransportLayer for MpscTransport {
        fn send(&self, message: Message) -> Result<(), ()> {
            let point = message;
            let serialized = serde_json::to_string(&point).unwrap();

            self.tx.send(serialized).unwrap();

            Ok(())
        }

        fn get(&self) -> Option<Message> {
            let deserialized: Message = serde_json::from_str(&self.rx.recv().unwrap()).unwrap();

            eprintln!("deserialized = {:?}", deserialized);
            Some(deserialized)
        }
    }

    impl MpscTransport {
        pub fn new() -> (MpscTransport, MpscTransport) {
            mpsc_transport_pair()
        }
    }

    fn mpsc_transport_pair() -> (MpscTransport, MpscTransport) {
        let (tx_h, rx_h) = std::sync::mpsc::channel();
        let (tx_d, rx_d) = std::sync::mpsc::channel();

        let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
        let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

        (host_channel, device_channel)
    }
}
