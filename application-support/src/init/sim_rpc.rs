//! TODO!

use super::{sim::new_sim, BlackBox, Init};
use crate::{
    event_loop::Backoff,
    shim_support::{new_shim_peripherals_set, Shims},
};

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{
    encoding::Transparent, futures::SyncEventFutureSharedState, mpsc_sync_pair,
    Controller, MpscTransport, RequestMessage, ResponseMessage,
};

use std::{sync::Mutex, thread::Builder as ThreadBuilder};

// Static data that we need:
// TODO: note that this will cause problems if more than 1 instance of this
// simulator is instantiated.
lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE_CONT: SyncEventFutureSharedState =
        SyncEventFutureSharedState::new();
}

type Cont<'ss> = Controller<
    'ss,
    MpscTransport<RequestMessage, ResponseMessage>,
    SyncEventFutureSharedState,
    RequestMessage,
    ResponseMessage,
    Transparent<RequestMessage>,
    Transparent<ResponseMessage>,
>;

pub struct SimWithRpcDevice<'ss> {
    controller: Cont<'ss>,
}

impl SimWithRpcDevice<'static> {}

impl<'s> Init<'s> for SimWithRpcDevice<'static> {
    type Config = ();

    type ControlImpl = Cont<'static>;
    type Input = SourceShim;
    type Output = Mutex<Vec<u8>>;

    fn init_with_config(
        b: &'s mut BlackBox,
        _config: Self::Config,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    ) {
        // Some of this is lifted verbatim from `src/init/sim.rs`:
        let input: &'static SourceShim = Box::leak(Box::new(SourceShim::new()));
        let output: &'static Mutex<Vec<u8>> =
            Box::leak(Box::new(Mutex::new(Vec::new())));

        let (shims, _, _) =
            new_shim_peripherals_set::<'static, 'static, _, _>(input, output);
        let shim_copy = Shims::from_peripheral_set(&shims);

        let (controller, device) = mpsc_sync_pair::<
            RequestMessage,
            ResponseMessage,
            Transparent<_>,
            Transparent<_>,
            Transparent<_>,
            Transparent<_>,
            _,
        >(&EVENT_FUTURE_SHARED_STATE_CONT);

        let _ = ThreadBuilder::new()
            .name("Device Thread".to_string())
            .stack_size(1024 * 1024 * 8)
            .spawn(move || {
                let mut sim = new_sim(shims);

                Backoff::default().run_step(&mut sim, device)
            })
            .unwrap();

        let storage: &'s mut _ = b.put(SimWithRpcDevice { controller });

        (
            &mut storage.controller,
            Some(shim_copy),
            Some(input),
            Some(output),
        )
    }
}
