//! The [`Snapshot` trait](crate::control::Snapshot).
//!
//! Allows for a type's state to be recorded and for a recorded state to
//! be restored.
//!
//! TODO!

use core::clone::Clone;
use core::convert::Infallible;
use core::fmt::{Debug, Display};

// TODO
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapshotError {
    UnrecordableState,
    UninterruptableState,
    Other(&'static str), // TODO: this should perhaps be a &dyn Debug or something
}

impl From<Infallible> for SnapshotError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Display for SnapshotError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        unimplemented!()
    }
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

    fn record(&self) -> Result<Self::Snap, Self::Err> {
        Ok(self.clone())
    }

    fn restore(&mut self, snap: Self) -> Result<(), Self::Err> {
        *self = snap;

        Ok(())
    }
}
