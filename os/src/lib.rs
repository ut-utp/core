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

        // use lc3_baseline_sim::mem_mapped::GPIO_OFFSET as OFS;

        // pub const INPUT_MODE: u8 = OFS + 0;
        // pub const OUTPUT_MODE: u8 = OFS + 1;
        // pub const INTERRUPT_MODE: u8 = OFS + 2;
        // pub const DISABLED_MODE: u8 = OFS + 3;

        // pub const READ: u8 = OFS + 4;
    }

    pub mod adc {
        // use lc3_baseline_sim::mem_mapped::ADC_OFFSET as OFS;

        define!([super::mm::ADC_OFFSET] <- {
            ENABLE,
            DISABLE,

            GET_MODE,

            READ,
        });
    }

    pub mod pwm {
        // use lc3_baseline_sim::mem_mapped::PWM_OFFSET as OFS;

        define!([super::mm::PWM_OFFSET] <- {
            ENABLE,
            DISABLE,

            GET_PERIOD,
            GET_DUTY,
        });
    }

    pub mod timers {
        // use lc3_baseline_sim::mem_mapped::TIMER_OFFSET as OFS;

        define!([super::mm::TIMER_OFFSET] <- {
            SINGLESHOT,
            REPEATED,
            DISABLE,

            GET_MODE,
            GET_PERIOD,
        });

    }

    pub mod clock {
        // use lc3_baseline_sim::mem_mapped::MISC_OFFSET as OFS;

        define!([super::mm::MISC_OFFSET] <- {
            SET,
            GET,
        });
    }

    pub mod input {
        pub use super::builtin::GETC as READ;
    }

    pub mod output {
        pub use super::builtin::OUT as WRITE;
    }

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
