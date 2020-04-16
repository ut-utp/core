//! Functions and bits that are useful for testing the interpreter.

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


#[doc(no_inline)]
pub use {
    lc3_isa::{insn, Addr, Instruction, Reg, Word},
    lc3_shims::{
        memory::MemoryShim,
        peripherals::{
            PeripheralsShim, ShareablePeripheralsShim, SourceShim
        },
    },
    lc3_baseline_sim::interp::{
        PeripheralInterruptFlags, Interpreter,
        InstructionInterpreterPeripheralAccess,
        InstructionInterpreter
    },
    lc3_application_support::shim_support::new_shim_peripherals_set
};

#[doc(no_inline)]
pub use pretty_assertions::*;


mod runner;
#[macro_use] pub mod macros;
mod misc;

pub use runner::*;
pub use misc::*;
