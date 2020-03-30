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

pub const USER_PROG_START_ADDR: lc3_isa::Addr = 0x0500;
pub const ERROR_ON_ACV_SETTING_ADDR: lc3_isa::Addr = 0x0501;

mod os;

/// Trap vector numbers.
pub mod traps {
    use lc3_baseline_sim::mem_mapped as mm;


    macro_rules! define {
        ([$starting:expr] <- { $first:ident $(,)? $($rest:ident$(,)?)* }) => {
            pub const $first: u8 = $starting;

            define!(munch $first $($rest)*);
        };

        (munch $previous:ident $next:ident $($rest:ident)*) => {
            pub const $next: u8 = $previous + 1;

            define!(munch $next $($rest)*);
        };

        (munch $previous:ident) => { }
    }

    /// Trap vectors for the [`Gpio`](lc3_traits::peripheral::Gpio) peripheral.
    pub mod gpio {
        define!([super::mm::GPIO_OFFSET] <- {
            INPUT_MODE,
            OUTPUT_MODE,
            INTERRUPT_MODE,
            DISABLED_MODE,

            GET_MODE,

            WRITE,
            READ,
        });
    }

    /// Trap vectors for the [`Adc`](lc3_traits::peripheral::Adc) peripheral.
    pub mod adc {
        define!([super::mm::ADC_OFFSET] <- {
            ENABLE,
            DISABLE,

            GET_MODE,

            READ,
        });
    }

    /// Trap vectors for the [`Pwm`](lc3_traits::peripheral::Pwm) peripheral.
    pub mod pwm {
        define!([super::mm::PWM_OFFSET] <- {
            ENABLE,
            DISABLE,

            GET_PERIOD,
            GET_DUTY,
        });
    }

    /// Trap vectors for the [`Timers`](lc3_traits::peripheral::Timers)
    /// peripheral.
    pub mod timers {
        define!([super::mm::TIMER_OFFSET] <- {
            SINGLESHOT,
            REPEATED,
            DISABLE,

            GET_MODE,
            GET_PERIOD,
        });

    }

    /// Trap vectors for the [`Clock`](lc3_traits::peripheral::Clock)
    /// peripheral.
    pub mod clock {
        define!([super::mm::MISC_OFFSET] <- {
            SET,
            GET,
        });
    }

    /// Trap vectors for the [`Input`](lc3_traits::peripheral::Input)
    /// peripheral.
    pub mod input {
        pub use super::builtin::GETC as READ;
    }

    /// Trap vectors for the [`Output`](lc3_traits::peripheral::Output)
    /// peripheral.
    pub mod output {
        pub use super::builtin::OUT as WRITE;
    }

    /// Trap vectors for the Traps officially part of the LC-3 ISA (i.e. GETC,
    /// OUT, PUTS, IN, HALT, etc.).
    pub mod builtin {
        define!([0x20] <- {
            GETC,   // 0x20
            OUT,    // 0x21
            PUTS,   // 0x22
            IN,     // 0x23
            PUTSP,  // 0x24
            HALT,   // 0x25
        });
    }
}

pub use os::{OS, OS_IMAGE};
