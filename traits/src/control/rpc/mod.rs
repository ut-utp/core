//! RPC for thingss that implement the [Control trait](super::Control).
//!
//! (TODO!)

//! For clarity, here's our whole picture:
//!
//! ```
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

pub fn new_pair<
    'a,
    'b,
    Req: Into<RequestMessage>,
    Resp: Into<ResponseMessage>,

    // Controller:
    ReqEnc: Encode<Req>, // = Transparent,
    RespDec: Decode<Resp>, // = Transparent,
    // Sends Requests, Receives Responses:
    ContTrans: Transport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
    S: EventFutureSharedState,

    // Device:
    ReqDec: Decode<Req>, // = Transparent,
    RespEnc: Encode<Resp>, // = Transparent,
    // Sends Responses, Receives Requests:
    DevTrans: Transport<<RespEnc as Encode>::Encoded, <ReqDec as Decode>::Encoded>,
    C: Control,
>(
        req_enc: ReqEnc,
        resp_dec: RespDec,
        cont_trans: ContTrans,
        state: &'a S,

        req_dec: ReqDec,
        resp_enc: RespEnc,
        dev_trans: DevTrans,
        _control_witness: Option<&'b C>,
) -> (
    Controller<
        'a,
        Req,
        Resp,
        ReqEnc,
        RespDec,
        ContTrans,
        S
    >,
    Device<
        Req,
        Resp,
        ReqDec,
        RespEnc,
        DevTrans,
        C
    >,
)
{
    let controller = Controller::new(req_enc, resp_dec, cont_trans, state);
    let device = Device::new(resp_enc, req_dec, dev_trans);

    (controller, device)
}

using_std! {
    use transport::MpscTransport;

    pub fn mpsc_pair<
        'a,
        Req: Into<RequestMessage>,
        Resp: Into<ResponseMessage>,

        // Controller:
        ReqEnc: Encode<Req>, // = Transparent,
        RespDec: Decode<Resp>, // = Transparent,
        S: EventFutureSharedState,

        // Device:
        ReqDec: Decode<Req>, // = Transparent,
        RespEnc: Encode<Resp>, // = Transparent,
        C: Control,
    >(
        req_enc: ReqEnc,
        resp_dec: RespDec,
        state: &'a S,

        req_dec: ReqDec,
        resp_enc: RespEnc,
    ) -> (
        Controller<
            'a,
            Req,
            Resp,
            ReqEnc,
            RespDec,
            // Sends Requests, Receives Responses:
            MpscTransport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
            S
        >,
        Device<
            Req,
            Resp,
            ReqDec,
            RespEnc,
            // Sends Responses, Receives Requests:
            MpscTransport<<RespEnc as Encode>::Encoded, <ReqDec as Decode>::Encoded>,
            C
    )
    {
        let (controller, device) = MpscTransport::new();

        new_pair(req_enc, resp_dec, controller, state, req_dec, resp_enc, device)
    }

    pub fn mpsc_sync_default_pair<
        'a,
        Req: Into<RequestMessage>,
        Resp: Into<ResponseMessage>,

        // Controller:
        ReqEnc: Default + Encode<Req>, // = Transparent,
        RespDec: Default + Decode<Resp>, // = Transparent,

        // Device:
        ReqDec: Default + Decode<Req>, // = Transparent,
        RespEnc: Default + Encode<Resp>, // = Transparent,
        C: Control,
    >(
            state: &'a SyncEventFutureSharedState
    ) -> (
        Controller<
            'a,
            Req,
            Resp,
            ReqEnc,
            RespDec,
            // Sends Requests, Receives Responses:
            MpscTransport<<ReqEnc as Encode>::Encoded, <RespDec as Decode>::Encoded>,
            S
        >,
        Device<
            Req,
            Resp,
            ReqDec,
            RespEnc,
            // Sends Responses, Receives Requests:
            MpscTransport<<RespEnc as Encode>::Encoded, <ReqDec as Decode>::Encoded>,
            C
    ) {
        use Default::default as def;
        mpsc_pair(def(), def(), state, def(), def())
    }
}
