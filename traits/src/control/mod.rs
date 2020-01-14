
// TODO: maybe rename this module:
pub mod control;
pub use control::{Event, State, Control};

pub mod metadata;
pub use metadata::{ProgramId, ProgramMetadata, Identifier, DeviceInfo, Capabilities, TypeIdExt, AnyExt};

pub mod snapshot;
pub use control::{Snapshot};

pub mod rpc;
