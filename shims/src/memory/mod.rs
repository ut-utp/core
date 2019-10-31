//! Memory shims.
//! (TODO)

pub mod error;

mod simple;
mod file_backed;

pub use simple::MemoryShim;
pub use file_backed::FileBackedMemoryShim;

