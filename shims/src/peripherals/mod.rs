//! Shims for the various [peripheral traits](crate::peripherals).

// Peripherals:
pub mod adc;
pub mod clock;
pub mod gpio;
pub mod pwm;
pub mod timers;

// Devices:
pub mod input;
pub mod output;

use lc3_traits::peripherals::PeripheralSet;

pub use adc::AdcShim;
pub use clock::ClockShim;
pub use gpio::GpioShim;
pub use pwm::PwmShim;
pub use timers::TimersShim;

pub use input::{InputShim, Source, SourceShim};
pub use output::{OutputShim, Sink};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, Mutex};

pub type ShareablePeripheralsShim<'int, 'io> = PeripheralSet<
    'int,
    Arc<RwLock<GpioShim<'int>>>,
    Arc<RwLock<AdcShim>>,
    Arc<Mutex<PwmShim>>,
    Arc<Mutex<TimersShim<'int>>>,
    Arc<RwLock<ClockShim>>,
    Arc<Mutex<InputShim<'io, 'int>>>,
    Arc<Mutex<OutputShim<'io, 'int>>>,
>;

sa::assert_impl_all!(ShareablePeripheralsShim<'_, '_>: Sync, Send);

// The assumption here is that your interrupt flags and input source/output sink
// live for the same amount of time (or, can be made to live for the same
// amount of time with lifetime sub-typing).
//
// This is usually (always?) fine; in the cases where it isn't you can ditch
// this type alias and spell out the type manually.
pub type PeripheralsShim<'s> = PeripheralSet<
    's,
    GpioShim<'s>,
    AdcShim,
    PwmShim,
    TimersShim<'s>,
    ClockShim,
    InputShim<'s, 's>,
    OutputShim<'s, 's>,
>;

#[derive(Debug)]
pub enum OwnedOrRefMut<'a, R: ?Sized> {
    Owned(Box<R>),
    Ref(&'a mut R),
}

impl<'a, R: ?Sized> Deref for OwnedOrRefMut<'a, R> {
    type Target = R;

    fn deref(&self) -> &R {
        use OwnedOrRefMut::*;

        match self {
            Owned(r) => r,
            Ref(r) => r,
        }
    }
}

impl<'a, R: ?Sized> DerefMut for OwnedOrRefMut<'a, R> {
    fn deref_mut(&mut self) -> &mut R {
        use OwnedOrRefMut::*;

        match self {
            Owned(r) => r,
            Ref(r) => r,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OwnedOrRef<'a, R: ?Sized> {
    Owned(Box<R>),
    Ref(&'a R),
}

impl<'a, R: ?Sized> Deref for OwnedOrRef<'a, R> {
    type Target = R;

    fn deref(&self) -> &R {
        use OwnedOrRef::*;

        match self {
            Owned(r) => r,
            Ref(r) => r,
        }
    }
}
