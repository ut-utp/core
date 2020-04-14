//! Shims for the various [peripheral traits](crate::peripherals).

// Peripherals:
mod adc;
mod clock;
mod gpio;
mod pwm;
mod timers;

// Devices:
mod input;
mod output;

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

pub type SharablePeripheralsShim<'int: 'io, 'io> = PeripheralSet<
    'int,
    Arc<RwLock<GpioShim<'int>>>,
    Arc<RwLock<AdcShim>>,
    Arc<Mutex<PwmShim>>,
    Arc<Mutex<TimersShim<'int>>>,
    Arc<RwLock<ClockShim>>,
    Arc<Mutex<InputShim<'io, 'int>>>,
    Arc<Mutex<OutputShim<'io, 'int>>>,
>;

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
