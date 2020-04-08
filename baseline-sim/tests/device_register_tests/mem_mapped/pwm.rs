use super::*;

use lc3_traits::peripherals::pwm::{Pwm, PwmPin, PwmState, PWM_PINS};
use lc3_baseline_sim::mem_mapped::{
    P0CR_ADDR, P0DR_ADDR,
    P1CR_ADDR, P1DR_ADDR,
};

use PwmState::*;
use PwmPin::*;

single_test! {
    get_initial_state,
    prefill: { 0x3010: P0CR_ADDR, },
    insns: [ { LDI R0, #0xF }, ],
    steps: 1,
    regs: { R0: 0x0000 },
    memory: { }
}

single_test! {
    get_initial_duty_cycle,
    prefill: { 0x3010: P0DR_ADDR, },
    insns: [ { LDI R0, #0xF }, ],
    steps: 1,
    regs: { R0: 0x0000 },
    memory: { }
}

single_test! {
    set_state,
    prefill: {
        0x3010: 0x00AB,
        0x3011: P0CR_ADDR,
    },
    insns: [
        { LD R0, #0xF },
        { STI R0, #0xF },
        { LDI R0, #0xE },
    ],
    steps: 3,
    regs: { R0: 0x00AB },
    memory: { }
}

single_test! {
    set_state_masks,
    prefill: {
        0x3010: 0xBEEF,
        0x3011: P0CR_ADDR,
    },
    insns: [
        { LD R0, #0xF },
        { STI R0, #0xF },
        { LDI R0, #0xE },
    ],
    steps: 3,
    regs: { R0: 0x00EF },
    memory: { }
}

single_test! {
    set_duty_cycle,
    prefill: {
        0x3010: 0x00AB,
        0x3011: P0DR_ADDR,
    },
    insns: [
        { LD R0, #0xF },
        { STI R0, #0xF },
        { LDI R0, #0xE },
    ],
    steps: 3,
    regs: { R0: 0x00AB },
    memory: { }
}

single_test! {
    set_duty_cycle_masks,
    prefill: {
        0x3010: 0xBEEF,
        0x3011: P0DR_ADDR,
    },
    insns: [
        { LD R0, #0xF },
        { STI R0, #0xF },
        { LDI R0, #0xE },
    ],
    steps: 3,
    regs: { R0: 0x00EF },
    memory: { }
}
