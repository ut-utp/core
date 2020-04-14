use super::*;

// TODO: test for correct output.
// currently only testing that the machine halts.

single_test! {
    out,
    pre: |p| { },
    prefill: { 0x3003: 33 },
    insns: [
        { LD R0, #2 },
        { TRAP #0x21 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| { },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    puts,
    pre: |p| { },
    prefill: {
        0x3003: 40,
        0x3004: 33,
        0x3005: 41,
        0x3006: 0
    },
    insns: [
        { LEA R0, #2 },
        { TRAP #0x22 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| { },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    putsp,
    pre: |p| { },
    prefill: {
        0x3003: 0x2128,
        0x3004: 0x0029,
        0x3006: 0
    },
    insns: [
        { LEA R0, #2 },
        { TRAP #0x24 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| { },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}
