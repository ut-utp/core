use super::*;

use lc3_traits::peripherals::timers::{Timers, TimerId, TimerState, TIMERS};
use lc3_baseline_sim::mem_mapped::{
    T0CR_ADDR, T0DR_ADDR, T0_INT_VEC,
    T1CR_ADDR, T1DR_ADDR, T1_INT_VEC,
};

use TimerState::*;
use TimerId::*;
