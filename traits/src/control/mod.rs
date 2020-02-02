// TODO: maybe rename this module:
pub mod control;
pub use control::{Control, Event, State};

pub mod metadata;
pub use metadata::{
    AnyExt, Capabilities, DeviceInfo, Identifier, ProgramId, ProgramMetadata, TypeIdExt,
};

pub mod snapshot;
pub use snapshot::{Snapshot, SnapshotError};

pub mod rpc;

// Ensure that the Control trait is Object Safe.
const _: Option<
    &dyn Control<EventFuture = rpc::EventFuture<'static, rpc::SimpleEventFutureSharedState>>,
> = None;
