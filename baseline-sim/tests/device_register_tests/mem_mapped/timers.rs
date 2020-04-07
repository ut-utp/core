use super::*;

use lc3_traits::peripherals::timers::{Timers, TimerId, TIMERS};
use lc3_baseline_sim::mem_mapped::{
    T0CR_ADDR, T0DR_ADDR,
    T1CR_ADDR, T1DR_ADDR,
    TIMER_BASE_INT_VEC, T0_INT_VEC
};

use TimerId::*;

single_test! {
    singleshot_100ms,
    prefill: {
        0x3010: T0CR_ADDR,
        0x3011: T0DR_ADDR,
        0x3012: 100,
        0x3013: 0,
    },
    prefill_expr: {
        (TIMER_BASE_INT_VEC): 0x3007,
    },
    insns: [
        { AND R0, R0, #0 }, // Mode: singleshot
        { STI R0, #0xE },   // Set to singleshot
        { LD R0, #0xF },    // Load period (100ms)
        { STI R0, #0xD },   // Set period to 100ms
        { LD R0, #0xE },    // Check if interrupt fired
        { BRnz #-2 },       // Go back one if not set
        { TRAP #0x25 },     // HALT

        { AND R1, R1, #0 },
        { ADD R1, R1, #1 }, // R1 <- #1
        { ST R1, #0xA },
        { RTI } // 0x300A
    ],
    regs: { },
    memory: { }
}
