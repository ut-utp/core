//! Memory shims.
//! (TODO)

pub mod error;

mod file_backed;
mod simple;

pub use file_backed::FileBackedMemoryShim;
pub use simple::MemoryShim;
