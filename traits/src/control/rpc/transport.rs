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

    // None if no messages were sent, Some(message) otherwise.
    fn get(&self) -> Result<RecvFormat, Self::RecvErr>; // TODO: should this be wrapped in a Result?
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

        fn get(&self) -> Result<Recv, Self::RecvErr> {
            let res = self.rx.try_recv();

            if let Ok(ref m) = res {
                log::trace!("GOT: {:?}", m);
            }

            res

            // TODO(fix): this breaks `run_until_event`!!
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
