//! Types and friends for the LC-3 ISA.
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

// Mark the crate as no_std if the `no_std` feature is enabled.
#![cfg_attr(feature = "no_std", no_std)]

use core::mem::size_of;

/// Address type/size for the LC-3.
pub type Addr = u16;

/// Maximum possible address value.
pub const ADDR_MAX_VAL: Addr = Addr::max_value();

/// Word type/size for the LC-3.
pub type Word = u16;

pub const PSR: Addr = 0xFFFC;
pub const MCR: Addr = 0xFFFE;

/// Maximum possible word value.
pub const WORD_MAX_VAL: Word = Word::max_value();

/// Size of the LC-3 address space in [`Word`](Word)s.
pub const ADDR_SPACE_SIZE_IN_WORDS: usize = (ADDR_MAX_VAL as usize) + 1;

/// Size of the LC-3 address space in bytes.
pub const ADDR_SPACE_SIZE_IN_BYTES: usize = ADDR_SPACE_SIZE_IN_WORDS * size_of::<Word>();

mod isa;

pub use isa::*;
