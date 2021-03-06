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

#[allow(unused_extern_crates)]
extern crate core; // makes rls actually look into the standard library (hack)

mod os;

pub mod traps;

pub use os::{OS, OS_IMAGE};
