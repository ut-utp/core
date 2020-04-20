//! TODO!

use super::{BlackBox, Init};
use crate::{
    shim_support::Shims,
};

use lc3_traits::control::rpc::{
    futures::SyncEventFutureSharedState,
    Controller, RequestMessage, ResponseMessage,
};
use lc3_device_support::{
    transport::uart_host::{HostUartTransport, SerialPortSettings},
    encoding::{PostcardEncode, PostcardDecode},
    util::Fifo,
};

use std::{
    sync::Mutex,
    thread::Builder as ThreadBuilder,
    path::Path,
    default::Default,
};

// Static data that we need:
// TODO: note that, like sim and sim_rpc, this will cause problems if more than
// 1 instance of this simulator is instantiated.
lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE_CONT: SyncEventFutureSharedState =
        SyncEventFutureSharedState::new();
}

type Cont<'ss, EncFunc: FnMut() -> Cobs<Fifo<u8>>> = Controller<
    'ss,
    HostUartTransport,
    SyncEventFutureSharedState,
    RequestMessage,
    ResponseMessage,
    PostcardEncode<RequestMessage, Cobs<Fifo<u8>>, EncFunc>,
    PostcardDecode<ResponseMessage, Cobs<Fifo<u8>>>,
>;

#[derive(Debug)]
pub struct BoardDevice<'ss, EncFunc: FnMut() -> Cobs<Fifo<u8>>> {
    controller: Cont<'ss, EncFunc>,
}

impl<'s, P, EncFunc> Init<'s> for BoardDevice<'static, EncFunc>
where
    P: AsRef<Path>,
    BoardConfig<P>: Default,
    EncFunc: FnMut() -> Cobs<Fifo<u8>>,
{
    type Config = BoardConfig<P>;

    type ControlImpl = Cont<'static, EncFunc>;
    type Input = SourceShim; // TODO
    type Output = Mutex<Vec<u8>>; // TODO

    fn init_with_config(
        b: &'s mut BlackBox,
        config: BoardConfig<P>,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    ) {
        let controller = Controller::new(
            PostcardEncode::with_fifo(),
            PostcardDecode::new(),
            config.new_transport(),
            &EVENT_FUTURE_SHARED_STATE_CONT
        );

        let storage: &'s mut _ = b.put(BoardDevice { controller });

        (
            &mut storage.controller,
            None,
            None, // TODO
            None, // TODO
        )
    }
}
