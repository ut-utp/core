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
mod device;

using_std! {
    use transport::{MpscTransport};

    pub fn mpsc_sync_pair<'a, Enc: Encoding + Default/* = TransparentEncoding*/, C: Control>(state: &'a SyncEventFutureSharedState) -> (Controller<'a, Enc, MpscTransport<Enc::Encoded>, SyncEventFutureSharedState>, Device<Enc, MpscTransport<Enc::Encoded>, C>)
    {
        let (controller, device) = MpscTransport::new();

        let controller = Controller::new(Enc::default(), controller, state);
        let device = Device::new(Enc::default(), device);

        (controller, device)
    }
}
