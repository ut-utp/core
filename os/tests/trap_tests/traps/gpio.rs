use super::*;

use lc3_traits::peripherals::gpio::{Gpio, GpioPin, GpioState, GPIO_PINS};
use lc3_baseline_sim::mem_mapped::{
    G0CR_ADDR, G0DR_ADDR, G0_INT_VEC,
    G1CR_ADDR, G1DR_ADDR, G1_INT_VEC,
    G2CR_ADDR, G2DR_ADDR, G2_INT_VEC,
    G3CR_ADDR, G3DR_ADDR, G3_INT_VEC,
    G4CR_ADDR, G4DR_ADDR, G4_INT_VEC,
    G5CR_ADDR, G5DR_ADDR, G5_INT_VEC,
    G6CR_ADDR, G6DR_ADDR, G6_INT_VEC,
    G7CR_ADDR, G7DR_ADDR, G7_INT_VEC,
    GPIODR_ADDR,
};

use GpioState::*;
use GpioPin::*;

mod states {
    use super::*;
}
