//! RPC for the Control trait.
//!
//! (TODO!)

use super::control::{Control, Event, State, MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use crate::error::Error as Lc3Error;
use crate::memory::MemoryMiscError;
use crate::peripherals::{adc::*, gpio::*, pwm::*, timers::*};
use lc3_isa::Reg;

use lc3_isa::*;

use core::task::Waker;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable};
use core::convert::Infallible;
use core::cell::Cell;
use core::num::NonZeroU8;
use core::sync::atomic::{AtomicBool, Ordering};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::fmt::Debug;

use serde::{Deserialize, Serialize};

// TODO: auto gen (proc macro, probably) the nice type up above from and the
// crimes below.


