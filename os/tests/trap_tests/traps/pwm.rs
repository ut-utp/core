use super::*;

use core::num::NonZeroU8;
use lc3_traits::peripherals::pwm::{Pwm, PwmPin, PwmState, PwmDutyCycle};
use PwmState::*;
use PwmPin::*;

single_test! {
    enable,
    prefill: {
        0x3005: 20,
        0x3006: 128
    },
    insns: [
        { AND R0, R0, #0 },
        { LD R1, #3 },
        { LD R2, #3 },
        { TRAP #0x50 },
        { TRAP #0x25 },
    ],
    post: |i| {
        let p = i.get_peripherals();
        eq!(Pwm::get_state(p, P0), Enabled(NonZeroU8::new(20).unwrap()));
        eq!(Pwm::get_duty_cycle(p, P0), 128);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    disable,
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x51 },
        { TRAP #0x25 },
    ],
    pre: |p| {
        Pwm::set_state(p, P0, Enabled(NonZeroU8::new(20).unwrap()));
        Pwm::set_duty_cycle(p, P0, 128);
    },
    post: |i| {
        let p = i.get_peripherals();
        eq!(Pwm::get_state(p, P0), Disabled);
        eq!(Pwm::get_duty_cycle(p, P0), 128);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_period,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x52 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    pre: |p| {
        Pwm::set_state(p, P0, Enabled(NonZeroU8::new(20).unwrap()));
        Pwm::set_duty_cycle(p, P0, 128);
    },
    post: |i| {
        eq!(i.get_word_unchecked(0x3004), 20);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_period_disabled,
    prefill: { 0x3004: 0xBEEF },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x52 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    post: |i| {
        eq!(i.get_word_unchecked(0x3004), 0);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_period_invalid_pin,
    prefill: { 0x3005: 0xBEEF },
    insns: [
        { AND R0, R0, #0 },
        { ADD R0, R0, #10 },
        { TRAP #0x52 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    post: |i| {
        eq!(i.get_word_unchecked(0x3005), 0);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}

single_test! {
    get_duty,
    prefill: { 0x3004: 0 },
    insns: [
        { AND R0, R0, #0 },
        { TRAP #0x53 },
        { ST R0, #1 },
        { TRAP #0x25 },
    ],
    pre: |p| {
        Pwm::set_state(p, P0, Enabled(NonZeroU8::new(20).unwrap()));
        Pwm::set_duty_cycle(p, P0, 128);
    },
    post: |i| {
        eq!(i.get_word_unchecked(0x3004), 128);
    },
    with os { MemoryShim::new(**OS_IMAGE) } @ OS_START_ADDR
}
