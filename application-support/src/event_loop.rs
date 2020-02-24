//! Functions that help applications run event loops around [`Control`] impls.
//!
//! ## No RPC Configurations
//!
//! In a no-rpc configuration, the application needs to call `Control::tick()`
//! often enough to actually make progress on any pending event futures.
//!
//! Assuming that the application doesn't have a separate execution thread, the
//! application needs to balance the following things on its main thread:
//!   - incoming events (i.e. user input, periodic update)
//!   - calling tick
//!
//! A naïve way to do this is to just call check for new events and call tick in
//! a loop. Like this:
//!
//! ```rust,ignore
//! loop {
//!     let e = loop {
//!         if let Ok(e) = recv.try_recv() { break e; }
//!         else {
//!             sim.tick()
//!         }
//!     };
//!
//!     handle_event(e);
//! }
//! ```
//!
//! The problem with this is that another goal of ours is to sleep when we have
//! nothing to do. With the solution above, we'd consume an entire core: even
//! when there are no new events and nothing actually happens in tick.
//!
//! The way we solved this for RPC setups was to have some kind of backoff
//! solution. `Control::tick` indicates to us whether it actually did any work
//! so if we discovered that no work was being done for a time, we'd sleep a
//! little (progressively longer for the longer it's been since we had real work
//! to do) before trying to do work again. This worked well enough for our RPC
//! setups.
//!
//! The problem with doing something like the above for _non-rpc_ setups is that
//! we do other things in the thread besides just calling `Control::tick`. If we
//! sleep, then other events won't be handled promptly. Put another way, the
//! amount we sleep by in the above approach reflects how much work the Control
//! impl is doing, but does *not* take into account whether we're receiving
//! events (we don't need a back off system for events since we can wait for
//! them instead of just polling for them). Ideally, we'd like to prioritize
//! events above sleeping while also not calling tick all the time.
//!
//! Conveniently there's a function on `mpsc::Receiver` that let's us block on
//! the next message _for some amount of time_ which let's us do exactly what we
//! want.
//!
//! Because we need to block on the next message in order to prioritize event
//! handling over sleeping/running tick, we need to do a little bit of an
//! inversion in how the control flow works. Rather than have the main loop call
//! us, a backoff/tick execution function, we will become the main loop and ask
//! for a function that we will call whenever events are sent and need to be
//! handled.
//!
//! ## RPC Configurations
//!
//! In an RPC configuration, things are actually simpler.
//!
//! #### Device Side
//!
//! The device thread of execution, free from having to do things other than
//! call step, can just call step and sleep.
//!
//! #### Controller Side
//!
//! The controller side still has a tick function to call, but it should never
//! _need_ to be called. The return value that the [`Controller`]'s
//! [`Control::tick`] indicates this. So, if we run such a [`Control`] impl
//! [exactly as we run a non-rpc `Control` impl](#no-rpc-configuration), the
//! event loop should quickly converge on basically just sleeping for the max
//! amount of time (allowing for events to interrupt this sleep) before calling
//! tick a few times. This is close to ideal (ideal being a pure blocking recv
//! call that puts the thread to sleep until messages come in).
//!
//!
//! [`Control`]: `lc3_traits::control::Control`
//! [`Controller`]: `lc3_traits::control::rpc::Controller`
//! [`Control::tick`]: `lc3_traits::control::Control::tick`

use lc3_traits::control::Control;
use lc3_traits::control::rpc::{RequestMessage, ResponseMessage, Decode, Encode, Transport, Device};

use std::fmt::Debug;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Backoff {
    pub num_iters: usize,
    pub idle_count_sleep_threshold: u64,
    pub idle_count_divisor: u64,
    pub max_sleep_time_ms: u64,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            num_iters: 100,
            idle_count_sleep_threshold: 500,
            idle_count_divisor: 100,
            max_sleep_time_ms: 100,
        }
    }
}

impl Backoff {
    // To be used on [`Control`] impls.
    //
    // func should return false to stop the loop and cause this function to
    // return.
    //
    // Returns `Ok(())` when stopped because `func` returned false and `Err(())`
    // when stopped because the Mpsc Receiver returned a Disconnected error.
    //
    // [`Control`]: `lc3_traits::control::Control`
    #[inline]
    pub fn run_tick_with_events<C: Control, E, F: FnMut(&mut C, E) -> bool>(&self, dev: &mut C, recv: Receiver<E>, mut func: F) -> Result<(), ()> {
        let mut idle_count = 0;

        loop {
            let insns: usize = (0..self.num_iters).map(|_| dev.tick()).sum();
            if insns == 0 { idle_count += 1; } else { idle_count = 0; }

            // If we're doing nothing, sleep for a bit or until messages come in.
            if idle_count > self.idle_count_sleep_threshold {
                let sleep_time = Duration::from_millis(self.max_sleep_time_ms.min(idle_count / self.idle_count_divisor));

                use RecvTimeoutError::*;
                match recv.recv_timeout(sleep_time) {
                    Ok(e) => if func(dev, e) { break Ok(()) },
                    Err(Timeout) => {},
                    Err(Disconnected) => break Err(()),
                }
            }

            // Once we get to the message processing phase, process any pending
            // messages before doing another spin:
            use TryRecvError::*;
            match loop {
                match recv.try_recv() {
                    Ok(e) => if func(dev, e) { break Ok(()) },
                    Err(e) => break Err(e),
                }
            } {
                Ok(()) => break Ok(()),
                Err(Disconnected) => break Err(()),
                Err(Empty) => {},
            }
        }
    }

    // To be used on the device side (i.e. on [`Device`]).
    //
    // Actually (embedded) devices will probably use a simpler spin loop than
    // this (since wasting cycles is less of a concern and also because this
    // requires OS functionality like `thread::sleep`).
    //
    // [`Device`]: `lc3_traits::control::rpc::Device`
    #[inline]
    pub fn run_step<C, Req, Resp, D, E, T>(&self, sim: &mut C, mut device: Device<T, C, Req, Resp, D, E>) -> !
    where
        Req: Debug,
        Resp: Debug,
        Req: Into<RequestMessage>,
        ResponseMessage: Into<Resp>,
        D: Decode<Req>,
        E: Encode<Resp>,
        T: Transport<<E as Encode<Resp>>::Encoded, <D as Decode<Req>>::Encoded>,
        C: Control,
        <C as Control>::EventFuture: Unpin, // TODO: use `pin_utils::pin_mut!` and relax this requirement. (see rpc::device)
   {
        let mut idle_count = 0;

        loop {
            let count: usize = (0..self.num_iters).map(|_| {
                let (msgs, insns) = device.step(sim);
                msgs + insns
            }).sum();

            if count == 0 { idle_count += 1; } else { idle_count = 0; }

            // If we're doing nothing, sleep.
            if idle_count > self.idle_count_sleep_threshold {
                let sleep_time = Duration::from_millis(self.max_sleep_time_ms.min(idle_count / self.idle_count_divisor));

                std::thread::sleep(sleep_time);
            }
        }
    }
}
