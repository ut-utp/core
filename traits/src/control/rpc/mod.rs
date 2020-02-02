//! RPC for things that implement the [Control trait](super::Control).
//!
//! (TODO!)

//! For clarity, here's our whole picture:
//!
//! ```text
//!   /----------------------------------------------------------------------\
//!  |                    [Controller Side: i.e. Laptop]                      |
//!  |                                                                        |
//!  |  /----------------------\                     %%% < %%%                |
//!  | | [Controller]: `Control`|               %%% [Main Loop] %%%           |
//!  | | tick:                  |                                             |
//!  | |  - resolves futures    |           %%%  /---------------\  %%%       |
//!  | |    issued by           |               |  [Client Logic] |           |
//!  | |    `run_until_event`   |<---\     %%%  |                 |   %%%     |
//!  | | rest:                  |    |     vvv  | Uses the device |   ^^^     |
//!  | |  - proxied; send req.  |    |     %%%  | via the Control |   %%%     |
//!  | |    and block on resp.  |    |          | interface.      |           |
//!  |  \--|----------------^--/     |     %%%  |  /---^          |  %%%      |
//!  |     |                |        |           \-|-------------/            |
//!  | |---v----|     |-----|---|    |        %%%  v              %%%         |
//!  | |Enc: Req|     |Dec: Resp|    \----------->[Control::tick]             |
//!  | |-|------|     |-------^-|                    %%% > %%%                |
//!   \--|--------------------|----------------------------------------------/
//!      |<Con Send  Con Recv>|
//!      |  [Transport Layer] |
//!      |<Dev Recv  Dev Send>|
//!   /--v--------------------|----------------------------------------------\
//!  | |--------|     |-------|-|            %%% < %%%            /--------\  |
//!  | |Dec: Req|     |Enc: Resp|       %%% [Dev. Loop] %%%      |  [Sim.]  | |
//!  | |---|----|     |-----^---|                       /--------| ╭──────╮ | |
//!  |     |                |       %%%                 |   %%%  | │Interp│ | |
//!  |  /--v----------------|--\                        |        | ╰──────╯ | |
//!  | |        [Device]        |  %%%                  v     %%% \--------/  |
//!  | | tick:                  |  vvv [Device::tick(......)] ^^^             |
//!  | |  - makes progress on   |  %%%     |                  %%%             |
//!  | |    any futures that    |<---------/                                  |
//!  | |    were issued         |  %%%                       %%%              |
//!  | |  - processes new reqs  |                                             |
//!  | |    (blocks if not a    |     %%%  v              %%%                 |
//!  | |    `run_until_event`)  |                                             |
//!  |  \----------------------/             %%% > %%%                        |
//!  |                                                                        |
//!  |                         [Device Side: i.e. TM4C]                       |
//!   \----------------------------------------------------------------------/
//! ```
//!
//! See the docs in the [device module](device) for more details.

use super::{Event, State, Control};

// TODO: Add tokio tracing to this (behind a feature flag) or just the normal
// log crate.

mod messages;
pub use messages::{RequestMessage, ResponseMessage};

pub mod encoding;
pub use encoding::{Encode, Decode};

pub mod transport;
pub use transport::Transport;

pub mod futures;
pub use futures::{EventFutureSharedState, EventFutureSharedStatePorcelain, SimpleEventFutureSharedState, EventFuture};

mod controller;
pub use controller::Controller;

mod device;
pub use device::Device;

use core::fmt::Debug;

pub fn new_pair<
    'a,
    'b,
    Req: Debug,
    Resp: Debug,

    // Controller:
    ReqEnc: Encode<Req>, // = Transparent,
    RespDec: Decode<Resp>, // = Transparent,
    // Sends Requests, Receives Responses:
    ContTrans: Transport<<ReqEnc as Encode<Req>>::Encoded, <RespDec as Decode<Resp>>::Encoded>,
    S: EventFutureSharedStatePorcelain,

    // Device:
    ReqDec: Decode<Req>, // = Transparent,
    RespEnc: Encode<Resp>, // = Transparent,
    // Sends Responses, Receives Requests:
    DevTrans: Transport<<RespEnc as Encode<Resp>>::Encoded, <ReqDec as Decode<Req>>::Encoded>,
    C: Control,
>(
        cont_trans: ContTrans,
        state: &'a S,

        dev_trans: DevTrans,
        _control_witness: Option<&'b C>,
) -> (
    Controller<
        'a,
        ContTrans,
        S,
        Req,
        Resp,
        ReqEnc,
        RespDec,
    >,
    Device<
        DevTrans,
        C,
        Req,
        Resp,
        ReqDec,
        RespEnc,
    >,
)
where
    RequestMessage: Into<Req>,
    Resp: Into<ResponseMessage>,

    Req: Into<RequestMessage>,
    ResponseMessage: Into<Resp>,

    // Controller's inputs (encoded responses) must = Device's outputs (encoded responses)
    RespDec: Decode<Resp, Encoded = <RespEnc as Encode<Resp>>::Encoded>,

    // Devices's inputs (encoded requests) must = Controller's outputs (encoded requests)
    ReqDec: Decode<Req, Encoded = <ReqEnc as Encode<Req>>::Encoded>,
{
    macro_rules! d { () => {Default::default()}; }

    let controller = Controller::new(d!(), d!(), cont_trans, state);
    let device = Device::new(d!(), d!(), dev_trans);

    (controller, device)
}

using_std! {
    use transport::MpscTransport;
    use futures::SyncEventFutureSharedState;


    pub fn mpsc_pair<
        'a,
        Req: Debug,
        Resp: Debug,

        // Controller:
        ReqEnc: Encode<Req>, // = Transparent,
        RespDec: Decode<Resp>, // = Transparent,
        S: EventFutureSharedStatePorcelain,

        // Device:
        ReqDec: Decode<Req>, // = Transparent,
        RespEnc: Encode<Resp>, // = Transparent,
        C: Control,
    >(
        state: &'a S,
    ) -> (
        Controller<
            'a,
            // Sends Requests, Receives Responses:
            MpscTransport<<ReqEnc as Encode<Req>>::Encoded, <RespDec as Decode<Resp>>::Encoded>,
            S,
            Req,
            Resp,
            ReqEnc,
            RespDec,
        >,
        Device<
            // Sends Responses, Receives Requests:
            MpscTransport<<RespEnc as Encode<Resp>>::Encoded, <ReqDec as Decode<Req>>::Encoded>,
            C,
            Req,
            Resp,
            ReqDec,
            RespEnc,
        >,
    )
    where
        RequestMessage: Into<Req>,
        Resp: Into<ResponseMessage>,

        Req: Into<RequestMessage>,
        ResponseMessage: Into<Resp>,

        // Controller's inputs (encoded responses) must = Device's outputs (encoded responses)
        RespDec: Decode<Resp, Encoded = <RespEnc as Encode<Resp>>::Encoded>,

        // Devices's inputs (encoded requests) must = Controller's outputs (encoded requests)
        ReqDec: Decode<Req, Encoded = <ReqEnc as Encode<Req>>::Encoded>,
    {
        let (controller, device) = MpscTransport::new();

        new_pair(controller, state, device, None)
    }

    pub fn mpsc_sync_pair<
        'a,
        Req: Debug,
        Resp: Debug,

        // Controller:
        ReqEnc: Encode<Req>, // = Transparent,
        RespDec: Decode<Resp>, // = Transparent,

        // Device:
        ReqDec: Decode<Req>, // = Transparent,
        RespEnc: Encode<Resp>, // = Transparent,
        C: Control,
    >(
            state: &'a SyncEventFutureSharedState
    ) -> (
        Controller<
            'a,
            // Sends Requests, Receives Responses:
            MpscTransport<<ReqEnc as Encode<Req>>::Encoded, <RespDec as Decode<Resp>>::Encoded>,
            SyncEventFutureSharedState,
            Req,
            Resp,
            ReqEnc,
            RespDec,
        >,
        Device<
            // Sends Responses, Receives Requests:
            MpscTransport<<RespEnc as Encode<Resp>>::Encoded, <ReqDec as Decode<Req>>::Encoded>,
            C,
            Req,
            Resp,
            ReqDec,
            RespEnc,
        >,
    )
    where
        RequestMessage: Into<Req>,
        Resp: Into<ResponseMessage>,

        Req: Into<RequestMessage>,
        ResponseMessage: Into<Resp>,

        // Controller's inputs (encoded responses) must = Device's outputs (encoded responses)
        RespDec: Decode<Resp, Encoded = <RespEnc as Encode<Resp>>::Encoded>,

        // Devices's inputs (encoded requests) must = Controller's outputs (encoded requests)
        ReqDec: Decode<Req, Encoded = <ReqEnc as Encode<Req>>::Encoded>,
    {
        mpsc_pair(state)
    }
}
