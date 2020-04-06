//! A barebones base image for the LC-3 with some tests.
//!
//! TODO!

#![recursion_limit = "2048"]
// TODO: forbid
#![warn(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_lifetimes,
    unused_comparisons,
    unused_parens,
    while_true
)]
// TODO: deny
#![warn(
    missing_debug_implementations,
    intra_doc_link_resolution_failure,
    missing_docs,
    unsafe_code,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    rust_2018_idioms
)]
#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!

#![deny(intra_doc_link_resolution_failure)] // TODO: this is temporary

// We're a no_std crate!
#![no_std]

// Note: this feature is not tested by CI (still dependent on nightly Rust) but
// this is fine for now.
#![cfg_attr(feature = "nightly-const", feature(const_if_match))]
#![cfg_attr(feature = "nightly-const", feature(const_panic))]
#![cfg_attr(feature = "nightly-const", feature(const_fn))]

// Makes some an item const if the nightly-const feature is activated and not
// const otherwise.
macro_rules! nightly_const {
    ([$($vis:tt)*] => [$($rest:tt)*]) => (
        #[cfg(not(feature = "nightly-const"))]
        $($vis)* $($rest)*

        #[cfg(feature = "nightly-const")]
        $($vis)* const $($rest)*
    );
}

extern crate static_assertions as sa;

// Note that these are 'page' aligned with the default starting sp pointing at
// the end of this page. The idea here is to minimize the number of pages that
// get modified (i.e. are dirty).

pub const USER_PROG_START_ADDR: lc3_isa::Addr = 0x0600;
pub const ERROR_ON_ACV_SETTING_ADDR: lc3_isa::Addr = 0x0601;
pub const OS_STARTING_SP_ADDR: lc3_isa::Addr = 0x0602;

pub const OS_DEFAULT_STARTING_SP: lc3_isa::Word = 0x0700;

mod os;

/// Trap vector numbers.
///
/// # Quick Reference Table
/// | Vector # | Name | Inputs | Outputs | Description |
/// | --- | --- | --- | --- | --- |
/// | `0x30` | GPIO_INPUT | `R0` - pin # |  `n` bit | Puts a GPIO pin in Input mode. |
/// | `0x31` | GPIO_OUTPUT | `R0` - pin # | `n` bit | Puts a GPIO pin in Output mode. |
/// | `0x32` | GPIO_INTERRUPT | `R0` - pin # <br>`R1` - address of ISR | `n` bit | Puts a GPIO in Interrupt mode and sets the ISR. |
/// | `0x33` | GPIO_DISABLED | `R0` - pin # | `n` bit | Puts a GPIO pin in Disabled mode. |
/// | `0x34` | GPIO_GET_MODE | `R0` - pin # | `R0` - GPIO mode <br>`n` bit | Returns the mode of a GPIO pin. |
/// | `0x35` | GPIO_WRITE | `R0` - pin # <br>`R1` - data to write | `n` bit | Writes to a GPIO pin in Output mode. |
/// | `0x36` | GPIO_READ | `R0` - pin # | `R0` - data from pin <br>`n` bit | Reads data from a GPIO pin. |
/// | `0x40` | ADC_ENABLE | `R0` - pin # | `n` bit | Puts an ADC pin in Enabled mode. |
/// | `0x41` | ADC_DISABLE | `R0` - pin # | `n` bit | Puts an ADC pin in Disabled mode. |
/// | `0x42` | ADC_GET_MODE | `R0` - pin # | `R0` - ADC mode <br>`n` bit | Returns the mode of an ADC pin. |
/// | `0x43` | ADC_READ | `R0` - pin # | `R0` - data from pin <br>`n` bit | Reads data from an ADC pin. |
/// | `0x50` | PWM_ENABLE | `R0` - pin # <br>`R1` - period <br>`R2` - duty cycle | `n` bit | Puts a PWM in Enabled mode, with period and duty cycle. |
/// | `0x51` | PWM_DISABLE | `R0` - pin # | `n` bit | Puts a PWM pin in Disabled mode. |
/// | `0x52` | PWM_GET_PERIOD | `R0` - pin # | `R0` - period <br>`n` bit | Returns the period of a PWM pin. |
/// | `0x53` | PWM_GET_DUTY | `R0` - pin # | `R0` - duty cycle <br>`n` bit | Returns the duty cycle of a PWM pin. |
/// | `0x60` | TIMER_SINGLESHOT | `R0` - id # <br>`R1` - period <br>`R2` - address of ISR | `n` bit | Puts a Timer in SingleShot mode with period and sets the ISR. |
/// | `0x61` | TIMER_REPEATED | `R0` - id # <br>`R1` - period <br> `R2` - address of ISR | `n` bit | Puts a Timer in Repeated mode with period and sets the ISR. |
/// | `0x62` | TIMER_DISABLE | `R0` - id # | `n` bit | Puts a Timer in Disabled mode. |
/// | `0x63` | TIMER_GET_MODE | `R0` - id # | `R0` - Timer mode <br>`n` bit | Returns the mode of a Timer. |
/// | `0x64` | TIMER_GET_PERIOD | `R0` - id # | `R0` - period <br>`n` bit | Returns the period of a Timer. |
/// | `0x70` | CLOCK_SET | `R0` - value to set | none | Sets the value of the Clock. |
/// | `0x71` | CLOCK_GET | none | `R0` - value of clock | Gets the value of the Clock. |
pub mod traps;

pub use os::{OS, OS_IMAGE};
