//! Traits and types for control's *transport* layer.
//!
//! TODO!

use crate::control::{Identifier, Version, version_from_crate};

use core::fmt::Debug;

pub trait Transport<SendFormat, RecvFormat> {
    type RecvErr: Debug;
    type SendErr: Debug;

    const ID: Identifier;
    const VER: Version;

    fn send(&self, message: SendFormat) -> Result<(), Self::SendErr>;


    /// Note: implementations of this method must *not* block.
    ///
    /// [`Device`](crate::control::rpc::Device)'s [step method](crate::control::rpc::Device::step)
    /// calls this function which is also where progress that isn't tied to
    /// messages is made (i.e. stepping while a `RunUntilEvent` request is
    /// active). If this function were to block, all progress would be tied to
    /// receiving messages and `run_until_event` would cease to work.
    ///
    /// Err(None) if no messages were sent, Ok(message) otherwise.
    /// Err(Some(_)) on transport errors.
    // TODO: went from Result<Option<M>, Err> to Result<M, Option<Err>>; is this
    // fine? is there a better way?
    fn get(&self) -> Result<RecvFormat, Option<Self::RecvErr>>;

    // Number of invalid/discarded messages.
    fn num_get_errors(&self) -> u64 { 0 }
}

using_std! {
    use std::sync::mpsc::{Sender, Receiver, SendError, TryRecvError};

    pub struct MpscTransport<SendFormat: Debug, RecvFormat: Debug> {
        tx: Sender<SendFormat>,
        rx: Receiver<RecvFormat>,
    }

    impl<Send: Debug, Recv: Debug> Transport<Send, Recv> for MpscTransport<Send, Recv> {
        type RecvErr = TryRecvError;
        type SendErr = SendError<Send>;

        const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("MPSC");
        const VER: Version = version_from_crate!();

        fn send(&self, message: Send) -> Result<(), Self::SendErr> {
            log::trace!("SENT: {:?}", message);
            self.tx.send(message)
        }

        fn get(&self) -> Result<Recv, Option<Self::RecvErr>> {
            let res = self.rx.try_recv();

            if let Ok(ref m) = res {
                log::trace!("GOT: {:?}", m);
            }

            res.map_err(|e| match e {
                TryRecvError::Empty => None,
                e => Some(e),
            })

            // this breaks `run_until_event`!!
            // This is because calls to Device::step (where progress is made)
            // can't happen anymore if we block here since tick calls this
            // function.
            //
            // This is unfortunate because we probably do want blocking calls on
            // the controller side. Foruntately, it's unlikely to spend much
            // time spinning on responses under normal use.
            //

            // Going to use this blocking variant for now even though it is likely to
            // result in worse performance for huge amounts of messages
            // let m = self.rx.recv().ok();
            // log::trace!("GOT: {:?}", m);
            // m
        }
    }

    fn mpsc_transport_pair<S: Debug, R: Debug>() -> (MpscTransport<S, R>, MpscTransport<R, S>) {
        let (tx_h, rx_h) = std::sync::mpsc::channel(); // S
        let (tx_d, rx_d) = std::sync::mpsc::channel(); // R

        let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
        let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

        (host_channel, device_channel)
    }

    impl<S: Debug, R: Debug> MpscTransport<S, R> {
        pub fn new() -> (MpscTransport<S, R>, MpscTransport<R, S>) {
            mpsc_transport_pair()
        }
    }
}
