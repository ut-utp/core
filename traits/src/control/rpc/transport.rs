//! Traits and types for control's *transport* layer.
//!
//! TODO!

pub trait Transport<SendFormat, RecvFormat> {
    type Err: Debug;

    fn send(&self, message: SendFormat) -> Result<(), Self::Err>;

    // None if no messages were sent, Some(message) otherwise.
    fn get(&self) -> Option<RecvFormat>; // TODO: should this be wrapped in a Result?
}

using_std! {
    use std::sync::mpsc::{Sender, Receiver, SendError};

    pub struct MpscTransport<SendFormat: Debug, RecvFormat: Debug> {
        tx: Sender<SendFormat>,
        rx: Receiver<RecvFormat>,
    }

    impl<Send: Debug, Recv: Debug> Transport<Send, Recv> for MpscTransport<Send, Recv> {
        type Err = SendError<EncodedFormat>;

        fn send(&self, message: Send) -> Result<(), Self::Err> {
            log::trace!("SENT: {:?}", message);
            self.tx.send(message)
        }

        fn get(&self) -> Option<Recv> {
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

    fn mpsc_transport_pair<S: Debug, R: Debug>() -> (MpscTransport<S, R>, MpscTransport<R, S>) {
        let (tx_h, rx_h) = std::sync::mpsc::channel(); // S
        let (tx_d, rx_d) = std::sync::mpsc::channel(); // R

        let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
        let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

        (host_channel, device_channel)
    }

    impl<EncodedFormat: Debug> MpscTransport<EncodedFormat> {
        pub fn new() -> (Self, Self) {
            mpsc_transport_pair()
        }
    }
}
