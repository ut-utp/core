use super::*;

use lc3_traits::peripherals::adc::{Adc, AdcPin, AdcState};
use lc3_shims::peripherals::AdcShim;

use AdcState::*;
use AdcPin::*;

single_test! {
    enable,
    pre: |p| { },
    prefill: { },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x40 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| {
        let p = i.get_peripherals();
        eq!(Adc::get_state(p, A0), Enabled);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    disable,
    pre: |p| { Adc::set_state(p, A0, Enabled); },
    prefill: { },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x41 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| {
        let p = i.get_peripherals();
        eq!(Adc::get_state(p, A0), Disabled);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_mode,
    pre: |p| { Adc::set_state(p, A0, Disabled); },
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x42 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 0); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_mode_enabled,
    pre: |p| { Adc::set_state(p, A0, Enabled); },
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x42 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 1); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    read,
    pre: |p| {
        Adc::set_state(p, A0, Enabled);
        AdcShim::set_value(p.get_adc(), A0, 10);
    },
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x43 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    regs: { },
    memory: { },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 10); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}
