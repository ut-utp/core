//! Supporting materials for devices running the UTP LC-3 Simulator.
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
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(html_logo_url = "")] // TODO!

#[allow(unused_extern_crates)]
extern crate core; // makes rls actually look into the standard library (hack)

#[macro_export]
macro_rules! not_wasm {
    ($($i:item)*) => {
        $( #[cfg(not(target_arch = "wasm32"))] $i )*
    };
}

#[macro_export]
macro_rules! wasm {
    ($($i:item)*) => {
        $( #[cfg(target_arch = "wasm32")] $i )*
    };
}

#[macro_export]
macro_rules! specialize {
    (   wasm: { $($wasm_item:item)+ }
        not: { $($other:item)+ }
    ) => {
        $crate::wasm! { $($wasm_item)+ }

        $crate::not_wasm! { $($other)+ }
    };
}

pub mod event_loop;
pub mod init;
pub mod io_peripherals;
pub mod shim_support;
