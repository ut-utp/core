//! Memory shims.
//! (TODO)

pub mod error;

not_wasm! {
    mod file_backed;
    pub use file_backed::FileBackedMemoryShim;
}

mod simple;
pub use simple::MemoryShim;
