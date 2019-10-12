#![feature(stmt_expr_attributes)]
#![feature(trace_macros)]
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
    unions_with_drop_fields,
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
    unused_results
)]
// Mark the crate as no_std if the `no_std` feature is enabled.
#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "shims")]
pub mod shims;

// Can't have `no_std` and `shims` enabled!
#[cfg(all(feature = "no_std", feature = "shims"))]
compile_error!(
    "Sorry! Can't provide shims for no_std targets. Either disable \
     the `no_std` feature or the `shims` feature."
);

pub type Addr = u16;
pub type Word = u16;

pub mod tests;

pub mod error;

pub mod control;
pub mod memory;
pub mod peripherals;

pub mod interp;
pub mod isa;
