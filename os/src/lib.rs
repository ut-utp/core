//! A barebones base image for the LC-3 with some tests.
//!
//! TODO!

#![recursion_limit = "1024"]

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

// Mark the crate as no_std if the `no_std` feature is enabled.
#![no_std]

pub const USER_PROG_START_ADDR: lc3_isa::Addr = 0x0400;
pub const ERROR_ON_ACV_SETTING_ADDR: lc3_isa::Addr = 0x0401;

mod os;

pub use os::{OS, OS_IMAGE};
