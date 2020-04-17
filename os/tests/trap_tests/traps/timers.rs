use super::*;

use lc3_traits::peripherals::timers::{Timers, TimerId, TimerState};

use TimerId::*;
use TimerState::*;

// TODO: flaky (race condition in shim/interpreter's interrupt handling?)
single_test! {
    singleshot,
    prefill: {
        0x300F: 200,
        0x3010: 0
    },
    insns: [
        { AND R0, R0, #0 },
        { LD R1, #0xD },
        { LEA R2, #4 },
        { TRAP #0x60 },

        { LD R1, #0xB }, // x3004
        { BRz #-2 },
        { TRAP #0x25 },

        { ADD R6, R6, #-1 }, // x3007
        { STR R0, R6, #0 },

        { AND R0, R0, #0 },
        { ADD R0, R0, #1 },
        { ST R0, #4 },

        { LDR R0, R6, #0 },
        { ADD R6, R6, #1 },
        { RTI } // x300E
    ],
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}


// TODO: check timing is correct. Seems too fast
single_test! {
    repeated,
    prefill: {
        0x3011: 200,
        0x3012: 0,
        0x3013: 0xFFF6,
    },
    insns: [
        { AND R0, R0, #0 },
        { LD R1, #0xF },
        { LEA R2, #6 },
        { TRAP #0x61 },

        { LD R1, #0xE }, // x3004
        { LD R0, #0xC },
        { ADD R0, R0, R1 },
        { BRn #-3 },
        { TRAP #0x25 },

        { ADD R6, R6, #-1 },
        { STR R0, R6, #0 },

        { LD R0, #6 },
        { ADD R0, R0, #1 },
        { ST R0, #4 },

        { LDR R0, R6, #0 },
        { ADD R6, R6, #1 },
        { RTI } // x3010
    ],
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

// TODO: check timing is correct. Seems too fast
single_test! {
    disable,
    prefill: {
        0x3015: 200,
        0x3016: 0,
        0x3017: 0xFFF6,
    },
    insns: [
        { AND R0, R0, #0 },
        { ADD R0, R0, #1 },
        { LD R1, #0x12 },
        { LEA R2, #9 },
        { TRAP #0x61 },

        { LD R1, #0x11 }, // x3004
        { LD R0, #0xF },
        { ADD R0, R0, R1 },
        { BRn #-3 },

        { AND R0, R0, #0 },
        { ADD R0, R0, #1 },
        { TRAP #0x62 },
        { TRAP #0x25 },

        { ADD R6, R6, #-1 },
        { STR R0, R6, #0 },

        { LD R0, #6 },
        { ADD R0, R0, #1 },
        { ST R0, #4 },

        { LDR R0, R6, #0 },
        { ADD R6, R6, #1 },
        { RTI } // x3013
    ],
    post: |i| {
        let p = i.get_peripherals();
        eq!(Timers::get_state(p, T1), Disabled);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_mode,
    prefill: {
        0x3008: 1000,
        0x3009: 1,
    },
    insns: [
        { AND R0, R0, #0 },
        { LD R1, #6 },
        { LEA R2, #4 },
        { TRAP #0x60 },
        { TRAP #0x63 },
        { ST R0, #3 },
        { TRAP #0x25 },
        { RTI }, // x3007
    ],
    post: |i| {
        eq!(i.get_word_unchecked(0x3009), 0);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_period,
    prefill: {
        0x3008: 1000,
        0x3009: 0,
    },
    insns: [
        { AND R0, R0, #0 },
        { LD R1, #6 },
        { LEA R2, #4 },
        { TRAP #0x60 },
        { TRAP #0x64 },
        { ST R0, #3 },
        { TRAP #0x25 },
        { RTI }, // x3007
    ],
    post: |i| {
        eq!(i.get_word_unchecked(0x3009), 1000);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}
