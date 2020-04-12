//! Test for the OS provided TRAPs.
//!
//! Assumes that the underlying memory mapped device registers work (they're
//! tested in the `lc3-baseline-sim` crate) and that the peripheral trait impls
//! used also work (the shims from the `lc3-peripheral-shims` crate are used;
//! they also have their own tests).

extern crate lc3_test_infrastructure as lti;
use lti::*;

use lti::Reg::*;

use lti::assert_eq as eq;

use lc3_baseline_sim::interp::InstructionInterpreter;
use lc3_os::OS_IMAGE;
use lc3_isa::OS_START_ADDR;

// mod adc;
mod clock;
mod gpio;
// mod pwm;
// mod timers;

// mod input;
// mod output;
