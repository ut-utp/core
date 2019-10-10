//! Shims for the various [peripheral](crate::peripherals) traits and for the
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
