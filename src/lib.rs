#![feature(stmt_expr_attributes)]
#![feature(trace_macros)]

// Mark the crate as no_std if the `no_std` feature is enabled.
#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "shims")]
mod shims;

// Can't have `no_std` and `shims` enabled!
#[cfg(all(feature = "no_std", feature = "shims"))]
compile_error!("Sorry! Can't provide shims for no_std targets. Either disable \
    the `no_std` feature or the `shims` feature.");

type Addr = u16;
type Word = u16;

mod error;

mod control;
mod memory;
mod peripherals;

mod isa;
mod interp;
