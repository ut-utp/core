//! Home of [an enum] that covers all the different Ranges in the standard
//! library.
//!
//! [an enum]: UnifiedRange

use core::ops::{
    Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    Bound, RangeBounds
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnifiedRange<T> {
    Range(Range<T>),
    RangeFrom(RangeFrom<T>),
    RangeFull(RangeFull),
    RangeInclusive(RangeInclusive<T>),
    RangeTo(RangeTo<T>),
    RangeToInclusive(RangeToInclusive<T>),
}

impl<T> RangeBounds<T> for UnifiedRange<T> {
    fn start_bound(&self) -> Bound<&T> {
        match self {
            Self::Range(r) => r.start_bound(),
            Self::RangeFrom(r) => r.start_bound(),
            Self::RangeFull(r) => r.start_bound(),
            Self::RangeInclusive(r) => r.start_bound(),
            Self::RangeTo(r) => r.start_bound(),
            Self::RangeToInclusive(r) => r.start_bound(),
        }
    }

    fn end_bound(&self) -> Bound<&T> {
        match self {
            Self::Range(r) => r.end_bound(),
            Self::RangeFrom(r) => r.end_bound(),
            Self::RangeFull(r) => r.end_bound(),
            Self::RangeInclusive(r) => r.end_bound(),
            Self::RangeTo(r) => r.end_bound(),
            Self::RangeToInclusive(r) => r.end_bound(),
        }
    }
}

macro_rules! into {
    ($($ty:tt)*) => {$(
        impl<T> From<$ty::<T>> for UnifiedRange<T> {
            fn from(r: $ty<T>) -> Self {
                UnifiedRange::$ty(r)
            }
        }
    )*};
}

impl<T> From<RangeFull> for UnifiedRange<T> {
    fn from(r: RangeFull) -> Self {
        Self::RangeFull(r)
    }
}

into! { Range RangeFrom RangeInclusive RangeTo RangeToInclusive }
