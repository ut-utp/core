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
//! _need_ to be called. The value that the [`Controller`]'s [`Control::tick`]
//! function returns indicates this (always returns 0). So, if we run such a
//! [`Control`] impl [exactly as we run a non-rpc `Control`
//! impl](#no-rpc-configuration), the event loop should quickly converge on
//! basically just sleeping for the max amount of time (allowing for events to
//! interrupt this sleep) before calling tick a few times. This is close to
//! ideal (ideal being a pure blocking recv call that puts the thread to sleep
//! until messages come in).
//!
//! ## WebAssembly
//!
//! On `wasm` targets we don't have threads. We also don't have `try_recv` (I
//! think `try_recv` might actually work but it definitely won't yield to the
//! event loop — just busy wait — which kind of defeats the point).
//!
//! Not having threads is a problem for us because our current approach of
//! blocking the thread (i.e. sleeping) until events are produced doesn't work
//! since, if we're blocking the current and only thread, nothing can produce
//! events.
//!
//! So, for `wasm` targets, we have to turn to `async` to help us out. The event
//! producer should give us something that we can `.await` on so that we can,
//! in the main loop, yield to the event loop until a timeout triggers or a
//! message arrives.
//!
//! [`Control`]: `lc3_traits::control::Control`
//! [`Controller`]: `lc3_traits::control::rpc::Controller`
//! [`Control::tick`]: `lc3_traits::control::Control::tick`

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

specialize! {
    wasm: {
        mod wasm;
        pub use wasm::timeout;
    }
    not: { mod not_wasm; }
}

use lc3_traits::control::Control;

/*
// To make sure that the `Backoff` interface is satisfied:
#[doc(hidden)]
fn __backoff_interface_check<
    S: ?Sized,
    P: for<'r> Fn(&'r mut S) -> &'r mut C,
    C: Control + ?Sized,
    E,
    F1: FnMut(&mut C, E) -> bool,
    F2: FnMut(&mut S, E) -> bool,
>() {
    let _ = Backoff::default();

    let _ = Backoff::run_tick_with_events::<C, E, F1>;
    let _ = Backoff::run_tick_with_event_with_project::<S, P, C, E, F2>;

    // not_wasm! {
    //     fn __run_step_check() { let _ = Backoff::run_step; }
    // }
}
*/
