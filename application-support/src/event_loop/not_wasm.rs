//! Implementation of the [`Backoff`] interface using threads and
//! `recv_timeout`; for platforms with threads (not WebAssembly).

// Note: any changes to this file have to be kept in sync with the wasm
// counterpart!

use super::Backoff;

use lc3_traits::control::Control;
use lc3_traits::control::rpc::{RequestMessage, ResponseMessage, Decode, Encode, Transport, Device};

use std::fmt::Debug;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::Duration;


impl Backoff {
    /// To be used on [`Control`] impls.
    ///
    /// `func` should return false to stop the loop and cause this function to
    /// return.
    ///
    /// Returns `Ok(())` when stopped because `func` returned false and
    /// `Err(())` when stopped because the Mpsc Receiver returned a Disconnected
    /// error.
    ///
    /// [`Control`]: `lc3_traits::control::Control`
    #[inline]
    pub fn run_tick_with_events<
        C: Control + ?Sized,
        E,
        F: FnMut(&mut C, E) -> bool
    >(
        &self,
        dev: &mut C,
        recv: Receiver<E>,
        func: F
    ) -> Result<(), ()> {
        self.run_tick_with_event_with_project::<C, _, C, E, F>(dev, |r| r, recv, func)
    }

    /// Just like [`run_tick_with_events`], but gives the event handling
    /// function access to a type that contains the [`Control`] impl.
    ///
    /// This is handy when the `Control` impl is part of a containing type and
    /// the event handling function needs access to the fields in the containing
    /// type. Because this function takes a mutable reference to the `Control`
    /// impl (as it must), with [`run_tick_with_events`] the event handling
    /// function is effectively prevented from accessing the containing type by
    /// the borrow checked since the mutable reference we ask for is held for
    /// the lifetime of this function.
    ///
    /// To get around this, this function asks for a mutable reference to the
    /// container type (`S` in the function signature) and a projection function
    /// to narrow `&mut S` into a mutable reference to the `Control` impl (`&mut
    /// C`).
    ///
    /// Because we relinquish access to the `Control` impl before we call the
    /// event handling function this works; the lifetimes of our mutable
    /// `Control` borrow and the event handling function's mutable borrow of `S`
    /// do *not* overlap.
    ///
    /// [`run_tick_with_events`]: `Backoff::run_tick_with_events`
    /// [`Control`]: `lc3_traits::control::Control`
    #[inline]
    pub fn run_tick_with_event_with_project<S, P, C, E, F>(
        &self,
        container: &mut S,
        project: P,
        recv: Receiver<E>,
        mut func: F,
    ) -> Result<(), ()>
    where
        S: ?Sized,
        P: for<'r> Fn(&'r mut S) -> &'r mut C,
        C: Control + ?Sized,
        F: FnMut(&mut S, E) -> bool,
    {
        let mut idle_count = 0;

        loop {
            let dev = project(container);
            let insns: usize = (0..self.num_iters).map(|_| dev.tick()).sum();
            if insns == 0 { idle_count += 1; } else { idle_count = 0; }

            // If we're doing nothing, sleep for a bit or until messages come in.
            if idle_count > self.idle_count_sleep_threshold {
                let sleep_time = Duration::from_millis(self.max_sleep_time_ms.min(idle_count / self.idle_count_divisor));

                use RecvTimeoutError::*;
                match recv.recv_timeout(sleep_time) {
                    Ok(e) => if !func(container, e) { break Ok(()) },
                    Err(Timeout) => {},
                    Err(Disconnected) => break Err(()),
                }
            }

            // Once we get to the message processing phase, process any pending
            // messages before doing another spin:
            use TryRecvError::*;
            match loop {
                match recv.try_recv() {
                    Ok(e) => if !func(container, e) { break Ok(()) },
                    Err(e) => break Err(e),
                }
            } {
                Ok(()) => break Ok(()),
                Err(Disconnected) => break Err(()),
                Err(Empty) => {},
            }
        }
    }

    /// To be used on the device side (i.e. on [`Device`]).
    ///
    /// Actual (i.e. embedded) devices will probably use a simpler spin loop
    /// than this (since wasting cycles is less of a concern and also because
    /// this requires OS functionality like `thread::sleep`).
    ///
    /// [`Device`]: `lc3_traits::control::rpc::Device`
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
