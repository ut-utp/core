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

// The order goes:
//
// -> LoadApiSession<PageWriteStart> -> Result<LoadApiSession<Index>, _>
//                       /----------------------->  |
//                      /                           v
//             LoadApiSession<Offset>  <--  LoadApiSession<Index>
//                      |                           |
//                      v                           v
//     send_page_chunk(...) /* moves */     finish(...) /* _consumes_ */

// Alternatively, to make it so that you can't start a new session without the
// old one being finished (this is a Trade Off):
//
// static TOKEN: Cell<Option<LoadApiSession<Empty>>> = Cell::new(Some(...));
//
//             LoadApiSession<Empty>
//                      |
//                      v
// LoadApiSession::new(..., page: PageIndex) -> Result<LoadApiSession<PageWriteStart>, _>
//                                                                 |
// /---------------------------------------------------------------/
// |
// \-> LoadApiSession<PageWriteStart> -> Result<LoadApiSession<Index>, _>
//                        /----------------------->  |
//                       /                           v
//              LoadApiSession<Offset>  <--  LoadApiSession<Index>
//                       |                           |
//                       v                           v  /* _consumes_ */
//      send_page_chunk(...) /* moves */     finish(...) -> Result<LoadApiSession<Empty>, LoadApiSession<Empty>>
//                                                                          |                    |
//                                                                          v                    V
//                                         (can be used to start a new session)    (return the same on error
//                                                                                    so you can start again)
//
impl LoadApiSession<PageWriteStart> {
    /// This is still unsafe since there's one major error case that we don't
    /// try to detect or report: calling other functions whilst in the middle of
    /// a load session.
    ///
    /// We still recommend that you just use [the porcelain](load_memory_dump) but if you
    /// want to do things manually, you must ensure that you do not call any
    /// other functions on [`Control`](crate::control::Control). If this
    /// invariant is not upheld, while memory safety issues will *not* occur
    /// (there is no _real_ unsafe code anywhere here), we make no guarantees
    /// about the data in [`Memory`](crate::memory::Memory).
    ///
    /// Additionally, we recommend you use [`load_memory_dump`] so you don't
    /// have to do the hashes yourself though note that computing the hashes
    /// wrong on the client side isn't an issue the way the previous thing
    /// (calling other functions during a load) is. At worst, you'll end up with
    /// the wrong data in the current page and not know it.
    #[allow(unsafe_code)]
    pub unsafe fn new(page: PageIndex) -> Result<LoadApiSession<PageWriteStart>, StartPageWriteError> {
        if page >= MEM_MAPPED_START_ADDR.page_idx() {
            Err(StartPageWriteError::InvalidPage { page })
        } else {
           Ok(LoadApiSession(PageWriteStart(page)))
        }
    }
}

impl LoadApiSession<PageIndex> {
    // NoCurrentSession isn't possible since you can only get the parent type
    // from a successfully started session.
    pub fn with_offset(&self, addr: Addr) -> Result<LoadApiSession<Offset>, PageChunkError> {
        if addr.page_idx() == self.0 {
            if let Some(last) = addr.checked_add((CHUNK_SIZE_IN_WORDS - 1) as Word) {
                if addr.page_idx() == last.page_idx() {
                    Ok(LoadApiSession(Offset(addr.page_offset())))
                } else {
                    Err(PageChunkError::ChunkCrossesPageBoundary { page: self.0, received_address: addr })
                }
            } else {
                // This should never happens since we don't allow writes to the
                // end of the address space (i.e. mem mapped space) anyways and
                // don't give you a `LoadApiSession<PageIndex>` if you try to do
                // this.
                unreachable!("Attempted to write past the end of the address space!");
            }
        } else {
            Err(PageChunkError::WrongPage { expected_page: self.0, received_address: addr })
        }
    }
}
