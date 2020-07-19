//! Implementation of the [`Backoff`] interface for WASM targets.

// Note: any changes to this file have to be kept in sync with the not_wasm
// counterpart!

use super::Backoff;

use lc3_traits::control::Control;
use lc3_traits::control::rpc::{RequestMessage, ResponseMessage, Decode, Encode, Transport, Device};

use futures_core::Stream;
use futures_util::{FutureExt, StreamExt, select};
use js_sys::{Function, Promise};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

use std::fmt::Debug;

/// An async timeout function for `wasm` that leans on the browser's
/// `setTimeout` function.
pub async fn timeout(window: &Window, ms: i32) {
    let promise = Promise::new(&mut |resolve, reject: Function| window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            &resolve,
            ms,
        )
        .map(|_| ())
        .unwrap_or_else(|err| {
            let _ = reject.call1(
                &JsValue::UNDEFINED,
                &err,
            ).unwrap();
        })
    );

    JsFuture::from(promise).await.unwrap();
}

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
    pub async fn run_tick_with_events<
        C: Control + ?Sized,
        E,
        R: Stream<Item = E>,
        F: FnMut(&mut C, E) -> bool
    >(
        &self,
        dev: &mut C,
        recv: R,
        func: F,
    ) -> Result<(), ()> {
        self
            .run_tick_with_event_with_project::<C, _, C, E, R, F>(dev, |r| r, recv, func)
            .await
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
    pub async fn run_tick_with_event_with_project<S, P, C, E, R, F>(
        &self,
        container: &mut S,
        project: P,
        recv: R,
        mut func: F
    ) -> Result<(), ()>
    where
        S: ?Sized,
        P: for<'r> Fn(&'r mut S) -> &'r mut C,
        C: Control + ?Sized,
        R: Stream<Item = E>,
        F: FnMut(&mut S, E) -> bool,
    {
        let mut idle_count = 0;

        // This isn't great. Ideally we'd have the caller pass this in but I
        // don't want to change the function signature _even more_ compared to
        // the non-wasm version.
        let window = web_sys::window().unwrap();
        let mut recv = Box::pin(recv.fuse());

        'outer: loop {
            let dev = project(container);
            let insns: usize = (0..self.num_iters).map(|_| dev.tick()).sum();
            if insns == 0 { idle_count += 1; } else { idle_count = 0; }

            // If we're doing nothing, sleep for a bit or until messages come in.
            if idle_count > self.idle_count_sleep_threshold {
                let sleep_time = self.max_sleep_time_ms.min(idle_count / self.idle_count_divisor) as i32;

                select! {
                    _ = timeout(&window, sleep_time).fuse() => { /* do another spin */ }
                    e = recv.next() => match e {
                        Some(e) => if !func(container, e) { break Ok(()) },
                        None => break Err(()),
                    }
                }
            }

            // Once we get to the message processing phase, process any pending
            // messages before doing another spin:
            'inner: loop {
                select! {
                    e = recv.next() => match e {
                        Some(e) => if !func(container, e) { break 'outer Ok(()) },
                        None => break 'outer Err(()),
                    },
                    // We don't want to wait; if there aren't events for us,
                    // break out of this loop.
                    default => break 'inner,
                }
            }
        }
    }
}
