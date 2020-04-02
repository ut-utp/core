use super::*;

use lc3_traits::peripherals::clock::Clock;
use lc3_baseline_sim::mem_mapped::CLKR_ADDR;
use std::thread::sleep;
use std::time::Duration;

single_test! {
    get_immediately,
    prefill: { 0x3010: CLKR_ADDR },
    insns: [ { LDI R0, #0xF } ],
    steps: 1,
    regs: { R0: 0x0000 },
    memory: { }
}

single_test! {
    get_after_100ms,
    pre: |p| { sleep(Duration::from_millis(100)); },
    prefill: { 0x3010: CLKR_ADDR },
    insns: [ { LDI R0, #0xF } ],
    steps: 1,
    regs: { R0: 100 },
    memory: { }
}

single_test! {
    reset,
    pre: |p| { sleep(Duration::from_millis(100)); },
    prefill: { 0x3010: CLKR_ADDR },
    insns: [
        { AND R0, R0, #0 },
        { STI R0, #0xF },
        { LDI R0, #0xF }
    ],
    steps: 3,
    regs: { R0: 0 },
    memory: { }
}

single_test! {
    set_higher,
    pre: |p| { sleep(Duration::from_millis(100)); },
    prefill: {
        0x3010: CLKR_ADDR,
        0x3011: 1000,
    },
    insns: [
        { LD R0, #0x10 },
        { STI R0, #0xF },
        { LDI R0, #0xF }
    ],
    steps: 4,
    regs: { R0: 1000 },
    memory: { }
}

