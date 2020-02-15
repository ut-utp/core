//! Types and friends for the `Future` that `Control::run_until_event` yields.
//!
//! TODO!

use crate::control::control::Event;

use core::task::{Waker, Context, Poll};
use core::future::Future;
use core::pin::Pin;
use core::cell::Cell;
use core::num::NonZeroU8;

// TODO: for now, getting multiple futures at once (i.e. calling run_until_event
// twice) is somewhat undefined: at least one of the futures will eventually
// resolve but there's no guarantee on which one it will be. Additionally, all
// (or just more than one) of the futures may also resolve.

/// All these functions take an immutable reference to self so that instances of
/// the implementor can be shared between the future and the Controller
/// implementation.
///
/// This trait does not require that implementors be Sync (a requirement imposed
/// on Futures by certain executors) but some implementations will be.
///
/// Implementors are encouraged to provide a const-fn constructor so that
/// instances of the implementation can be put into `static` variables and
/// therefore be `'static` (this is desirable since most executors - at least
/// those that don't use scoped thread pools - require that the futures they
/// execute be `'static`).
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

// We don't provide this blanket impl because the default implementation of the
// porcelain trait makes multiple calls on `EventFutureSharedState`; for implementations
// of `EventFutureSharedState` that accquire locks this can be problematic.
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

        match self {
            Errored | Dormant => panic!("Unregistered future polled the state! {:?}", self),
            WaitingForFuturesToResolve { waker: Some(_), .. } => panic!("Waker persisted after batch was sealed!"),
            WaitingForAnEvent { .. } => None,
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
        }
    }

    fn increment(&mut self) -> u8 {
        use SharedStateState::*;

        match self {
            Errored => unreachable!(),
            WaitingForFuturesToResolve { .. } => panic!("Attempted to add a future to a sealed batch!"),
            Dormant => {
                *self = WaitingForAnEvent { waker: None, count: NonZeroU8::new(1).unwrap() };
                1
            },
            WaitingForAnEvent { count, .. } => {
                // println!("another future!!");
                *count = NonZeroU8::new(count.get().checked_add(1).unwrap()).unwrap();
                count.get()
            }
        }
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
        assert!(self.is_clean(), "Tried to reset before all Futures resolved!");

        *self = SharedStateState::Dormant;
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
pub struct EventFuture<'a, S: EventFutureSharedState>(pub &'a S);

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

using_std! {
    use std::sync::RwLock;

    #[derive(Debug)]
    pub struct SyncEventFutureSharedState(RwLock<SharedStateState>);

    impl SyncEventFutureSharedState {
        // This, unfortunately, can't be const. Users will need to use
        // lazy_static or something similar.
        pub fn new() -> Self {
            Self(RwLock::new(SharedStateState::Dormant))
        }
    }

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
}
