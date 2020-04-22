// TODO: maybe rename this module:

// `metadata` has to come before `control` because of the macro it contains.
pub mod metadata;
pub use metadata::{
    AnyExt, Capabilities, DeviceInfo, Identifier, ProgramId, ProgramMetadata, TypeIdExt,
    Version, version_from_crate
};

pub mod control;
pub use control::{Control, Event, State, ProcessorMode};

pub mod ext;

pub mod load;
pub use load::{load_memory_dump, Progress};

pub mod snapshot;
pub use snapshot::{Snapshot, SnapshotError};

pub mod ranges;
pub use ranges::UnifiedRange;

pub mod rpc;

// Ensure that the Control trait is Object Safe.
const _: Option<
    &dyn Control<EventFuture = rpc::EventFuture<'static, rpc::SimpleEventFutureSharedState>>,
> = None;
