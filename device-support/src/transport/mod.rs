//! TODO!

use super::util::fifo::{self, Fifo};

use lc3_traits::control::rpc::{RequestMessage, ResponseMessage};

use core::mem::size_of;


// Check that CAPACITY is such that we can hold at least one full
// request/response:
sa::const_assert!(fifo::CAPACITY >= size_of::<RequestMessage>());
sa::const_assert!(fifo::CAPACITY >= size_of::<ResponseMessage>());
