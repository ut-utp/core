//! Types and friends that are part of the load API (a part of the
//! [`Control` trait](crate::control::Control).)
//!
//! TODO!

use super::Control;

use lc3_isa::{Addr, Word, Bits, util::MemoryDump, MEM_MAPPED_START_ADDR, ADDR_MAX_VAL, ADDR_SPACE_SIZE_IN_WORDS};

use core::mem::size_of;
use core::hash::{Hash, Hasher};
#[allow(deprecated)] use core::hash::SipHasher;
use core::sync::atomic::{AtomicUsize, Ordering::{SeqCst, Relaxed}};
use core::time::Duration;
use core::convert::TryInto;

use serde::{Deserialize, Serialize};

pub type PageIndex = u8;
pub type PageOffset = u8;
sa::const_assert_eq!(size_of::<PageIndex>() + size_of::<PageOffset>(), size_of::<Word>());

pub const PAGE_SIZE_IN_WORDS: Addr = (PageOffset::max_value() as Addr) + 1;
pub const NUM_PAGES: usize = (ADDR_SPACE_SIZE_IN_WORDS) / (PAGE_SIZE_IN_WORDS as usize);
pub const NUM_MEM_MAPPED_PAGES: usize = (PageIndex::max_value() - (MEM_MAPPED_START_ADDR >> 8) as PageIndex) as usize + 1;

pub const CHUNK_SIZE_IN_WORDS: PageOffset = 8;
pub const CHUNKS_IN_A_PAGE: usize = (PAGE_SIZE_IN_WORDS as usize) / (CHUNK_SIZE_IN_WORDS as usize);

// TODO: Ideally this would take a reference to an array and not a slice, but alas
//
// TODO: perhaps switch to something better suited to being a checksum
pub fn hash_page(page: &[Word]) -> u64 {
    #[allow(deprecated)]
    let mut hasher = SipHasher::new(); // TODO: deprecated but what can we do...

    Word::hash_slice(page, &mut hasher);
    hasher.finish()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Index(PageIndex);

impl Index {
    pub const fn with_offset(&self, offset: PageOffset) -> Addr {
        (self.0 as Addr) << 8 + (offset as Addr)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Offset(PageOffset);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StartPageWriteError {
    InvalidPage { page: u8 }, // Only the mem-mapped page should be invalid...
    UnfinishedSessionExists { unfinished_page: PageIndex },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageChunkError {
    NoCurrentSession,
    WrongPage { expected_page: PageIndex, received_address: Addr, },
    ChunkCrossesPageBoundary { page: PageIndex, received_address: Addr, },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FinishPageWriteError {
    NoCurrentSession,
    SessionMismatch { current_session_page: PageIndex, received_page: PageIndex },
    ChecksumMismatch { page: PageIndex, given_checksum: u64, computed_checksum: u64 },
}

// Need this newtype to have `LoadApiSession<Index>` be different than
// `LoadApiSession<Start>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PageWriteStart(pub PageIndex);

// Private field so users can't construct this manually.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadApiSession<State>(State);

