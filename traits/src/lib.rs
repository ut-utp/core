//! Traits defining the LC-3's peripherals, memory, and control interface.
//!
//! TODO!

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
#![cfg_attr(feature = "no_std", no_std)]

// Can't have `no_std` and `std_features` enabled!
#[cfg(all(feature = "no_std", feature = "std_functionality"))]
compile_error!(
    "Sorry! Can't provide std functionality for no_std targets. Either disable \
     the `no_std` feature or the `std_functionality` feature."
);

macro_rules! using_std { ($($i:item)*) => ($(#[cfg(feature = "std_functionality")]$i)*) }

pub mod test_infrastructure;

pub mod error;

pub mod control;
pub mod memory;
pub mod peripherals;
