//! Home of [an enum] that covers all the different Ranges in the standard
//! library.
//!
//! [an enum]: UnifiedRange

use core::ops::{
    Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    Bound as CoreBound, RangeBounds
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnifiedRange<T> {
    start: Bound<T>,
    end: Bound<T>,
}

// A copy of `core::ops::Bound` that we can serialize (and turn into
// `core::ops::Bound` instances).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Bound<T> {
    Includes(T),
    Excludes(T),
    Unbounded
}

impl<T: Clone> From<CoreBound<&T>> for Bound<T> {
    #[inline]
    fn from(b: CoreBound<&T>) -> Self {
        match b {
            CoreBound::Included(b) => Bound::Includes(b.clone()),
            CoreBound::Excluded(b) => Bound::Excludes(b.clone()),
            CoreBound::Unbounded => Bound::Unbounded,
        }
    }
}

impl<T> Bound<T> {
    #[inline]
    fn as_core_bound(&self) -> CoreBound<&T> {
        match self {
            Bound::Includes(b) => CoreBound::Included(b),
            Bound::Excludes(b) => CoreBound::Excluded(b),
            Bound::Unbounded => CoreBound::Unbounded,
        }
    }
}

impl<T> RangeBounds<T> for UnifiedRange<T> {
    fn start_bound(&self) -> CoreBound<&T> {
        self.start.as_core_bound()
    }

    fn end_bound(&self) -> CoreBound<&T> {
        self.end.as_core_bound()
    }
}

macro_rules! into {
    ($($ty:tt)*) => {$(
        impl<T: Clone> From<$ty::<T>> for UnifiedRange<T> {
            fn from(r: $ty<T>) -> Self {
                UnifiedRange {
                    start: r.start_bound().into(),
                    end: r.end_bound().into(),
                }
            }
        }
    )*};
}

impl<T: Clone> From<RangeFull> for UnifiedRange<T> {
    fn from(r: RangeFull) -> Self {
        Self {
            start: r.start_bound().into(),
            end: r.end_bound().into(),
        }
    }
}

into! { Range RangeFrom RangeInclusive RangeTo RangeToInclusive }
