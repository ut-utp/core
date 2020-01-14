//! The [`Snapshot` trait](crate::control::Snapshot).
//!
//! Allows for a type's state to be recorded and for a recorded state to
//! be restored.
//!
//! TODO!

pub trait Snapshot {
    type Snap;

    fn record(&self) -> Self::Snap;
    fn restore(&mut self, snap: Self::Snap);
}

using_std! {
    use std::clone::Clone;
    impl<T: Clone> Snapshot for T {
        type Snap = Self;

        fn record(&self) -> Self {
            self.clone()
        }

        fn restore(&mut self, snap: Self) {
            *self = snap;
        }
    }
}
