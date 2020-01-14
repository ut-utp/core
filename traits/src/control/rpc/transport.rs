//! Traits and types for control's *transport* layer.
//!
//! TODO!

pub trait Transport<EncodedFormat> {
    type Err: Debug;

    fn send(&self, message: EncodedFormat) -> Result<(), Self::Err>;

    // None if no messages were sent, Some(message) otherwise.
    fn get(&self) -> Option<EncodedFormat>; // TODO: should this be wrapped in a Result?
}

using_std! {
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
