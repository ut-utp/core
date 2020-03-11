//! Utils and impls for the [`Memory` trait](Memory).
//!
//! TODO!
//! TODO!
//! [Memory]: lc3_traits::memory::Memory

pub mod simple;
pub use simple::PartialMemory;

pub mod paged;
pub use paged::FlashBackedMemory;
