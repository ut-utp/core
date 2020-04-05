//! Tests for memory mapped registers!
//!
//! Also kind of ends up testing that the peripherals used work (the shims do
//! have their own unit tests anyways though).

use lc3_test_infrastructure::*;

use lc3_isa::SignedWord;
use Reg::*;

use itertools::Itertools;
use lc3_test_infrastructure::assert_eq as eq;

mod adc;
mod clock;
mod gpio;
mod pwm;
mod timers;

mod input;
mod output;
