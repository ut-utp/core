use super::*;

use lc3_traits::peripherals::adc::{Adc, AdcPin, AdcState};
use lc3_shims::peripherals::AdcShim;

use AdcState::*;
use AdcPin::*;

use std::sync::RwLock;

single_test! {
    enable,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x40 },
        { TRAP #0x25 },
    ],
    post: |i| {
        let p = i.get_peripherals();
        eq!(Adc::get_state(p, A0), Enabled);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    disable,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x41 },
        { TRAP #0x25 },
    ],
    pre: |p| { Adc::set_state(p, A0, Enabled); },
    post: |i| {
        let p = i.get_peripherals();
        eq!(Adc::get_state(p, A0), Disabled);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_mode,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x42 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    pre: |p| { Adc::set_state(p, A0, Disabled); },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 0); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_mode_enabled,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x42 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    pre: |p| { Adc::set_state(p, A0, Enabled); },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 1); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    read,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x43 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    pre: |p| {
        Adc::set_state(p, A0, Enabled);
        AdcShim::set_value(&mut *p.get_adc().write().unwrap(), A0, 10);
    },
    post: |i| { eq!(i.get_word_unchecked(0x3004), 10); },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}
