use super::*;

use lc3_traits::peripherals::timers::{Timers, TimerId, TIMERS};
use lc3_baseline_sim::mem_mapped::{
    MemMapped,
    T0CR_ADDR, T0DR_ADDR,
    T1CR_ADDR, T1DR_ADDR,
    TIMER_BASE_INT_VEC, T0_INT_VEC,
    PSR,
    MCR
};

use TimerId::*;

single_test! {
    singleshot_100ms,
    prefill: {
        0x3010: 0x0700,
        0x3011: T0CR_ADDR,
        0x3012: T0DR_ADDR,
        0x3013: 100,
        0x3014: <MCR as MemMapped>::ADDR,
        0x3015: 0,
    },
    prefill_expr: {
        (TIMER_BASE_INT_VEC): 0x3009,
        (<PSR as MemMapped>::ADDR): 0x0302,
    },
    insns: [
        { LD R6, #0xF },    // Set nonzero R6
        { AND R0, R0, #0 }, // Mode: singleshot
        { STI R0, #0xE },   // Set to singleshot
        { LD R0, #0xF },    // Load period (100ms)
        { STI R0, #0xD },   // Set period to 100ms
        { LD R0, #0xF },    // Check if interrupt fired
        { BRnz #-2 },       // Go back one if not set

        { AND R0, R0, #0 }, // Prep HALT
        { STI R0, #0xB },   // HALT (0x3008)

        { AND R1, R1, #0 },
        { ADD R1, R1, #1 }, // R1 <- #1
        { ST R1, #9 },
        { RTI } // 0x300C
    ],
    regs: { },
    memory: { }
}

// TODO: flaky (sometimes hangs)
single_test! {
    repeated_100ms,
    prefill: {
        0x3010: 0x0700,
        0x3011: T0CR_ADDR,
        0x3012: T0DR_ADDR,
        0x3013: 100,
        0x3014: <MCR as MemMapped>::ADDR,
        0x3015: 0xFFFC, // -4
    },
    prefill_expr: {
        (TIMER_BASE_INT_VEC): 0x3009,
        (<PSR as MemMapped>::ADDR): 0x0302,
    },
    insns: [
        { LD R6, #0xF },    // Set nonzero R6
        { AND R0, R0, #0 },
        { ADD R0, R0, #1 }, // Mode: repeated
        { STI R0, #0xD },   // Set to repeated
        { LD R0, #0xE },    // Load period (100ms)
        { STI R0, #0xC },   // Set period to 100ms
        { LD R0, #0xE },    // Check if interrupt fired five times
        { BRnz #-2 },       // Go back one if not

        { AND R0, R0, #0 }, // Prep HALT
        { STI R0, #0xA },   // HALT (0x3009)

        { LD R1, #0xA },
        { ADD R1, R1, #1 },
        { ST R1, #9 },
        { RTI } // 0x300D
    ],
    regs: { },
    memory: { }
}
