//! An instruction level simulator for the LC-3.
//!
//! TODO!

// #![feature(try_trait)]

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

#![warn(clippy::missing_docs_in_private_items)] // TODO: add to all

#![doc(test(attr(deny(warnings))))]
#![doc(html_logo_url = "")] // TODO!
// TODO: add doc URL to all

// Mark the crate as no_std if the `no_std` feature is enabled.
#![cfg_attr(feature = "no_std", no_std)]

extern crate static_assertions as sa;


pub mod interp;
pub mod mem_mapped;
pub mod sim;

pub use mem_mapped::*;
