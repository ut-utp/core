//! The [`Snapshot` trait](crate::control::Snapshot).
//!
//! Allows for a type's state to be recorded and for a recorded state to
//! be restored.
//!
//! TODO!

use core::clone::Clone;
use core::convert::Infallible;

// TODO
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapshotError {
    UnrecordableState,
    UninterruptableState,
    Other(&'static str),
}

impl Display for SnapshotError {
    // TODO!
}

using_std! { impl std::error::Error for SnapshotError { } }

pub trait Snapshot {
    type Snap;
    type Err: Debug + Into<SnapshotError>;

    // Is fallible; can fail if the simulator is in a state that can't be
    // snapshotted.
    fn record(&self) -> Result<Self::Snap, Self::Err>;

    // This is also fallible. This can fail if the simulator is not in a state
    // where the current state can be abandoned for an old state.
    fn restore(&mut self, snap: Self::Snap) -> Result<(), Self::Err>;
}

impl<T: Clone> Snapshot for T {
    type Snap = Self;
    type Err = Infallible;

    fn record(&self) -> Self {
        self.clone()
    }

    fn restore(&mut self, snap: Self) {
        *self = snap;
    }
}
