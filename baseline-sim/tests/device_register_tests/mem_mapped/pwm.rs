use super::*;

use lc3_traits::peripherals::pwm::{Pwm, PwmPin, PwmState, PWM_PINS};
use lc3_baseline_sim::mem_mapped::{
    P0CR_ADDR, P0DR_ADDR,
    P1CR_ADDR, P1DR_ADDR,
};

use PwmState::*;
use PwmPin::*;
