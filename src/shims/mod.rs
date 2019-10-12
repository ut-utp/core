//! Shims for the various [peripheral traits](crate::peripherals) and for the
//! [`Memory` trait](crate::memory::Memory).

// Peripherals:
mod adc;
mod clock;
mod gpio;
mod pwm;
mod timers;

// Devices:
mod input;
mod output;

// Memory:
mod memory;

use crate::peripherals::{PeripheralSet, Peripherals};

pub use adc::AdcShim;
pub use clock::ClockShim;
pub use gpio::GpioShim;
pub use pwm::PwmShim;
pub use timers::TimersShim;

pub use input::InputShim;
pub use output::OutputShim;

pub use memory::MemoryShim;

pub type PeripheralsShim<'s> =
    PeripheralSet<'s, GpioShim<'s>, AdcShim, PwmShim, TimersShim, ClockShim, InputShim, OutputShim>;

// impl Peripherals for PeripheralsShim {
//     fn init() -> Self {

//     }
// }
