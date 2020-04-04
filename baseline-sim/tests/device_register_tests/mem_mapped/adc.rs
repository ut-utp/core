use super::*;

use lc3_traits::peripherals::adc::{Adc, AdcPin, AdcState, ADC_PINS};
use lc3_baseline_sim::mem_mapped::{
    A0CR_ADDR, A0DR_ADDR,
    A1CR_ADDR, A1DR_ADDR,
    A2CR_ADDR, A2DR_ADDR,
    A3CR_ADDR, A3DR_ADDR,
    A4CR_ADDR, A4DR_ADDR,
    A5CR_ADDR, A5DR_ADDR,
};

use AdcState::*;
use AdcPin::*;
