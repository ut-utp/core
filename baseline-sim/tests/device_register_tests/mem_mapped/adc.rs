use super::*;

use lc3_traits::peripherals::adc::{Adc, AdcPin, AdcState, ADC_PINS};
use lc3_baseline_sim::mem_mapped::{
    A0CR_ADDR, A0DR_ADDR,
    A1CR_ADDR, A1DR_ADDR,
    A2CR_ADDR, A2DR_ADDR,
    A3CR_ADDR, A3DR_ADDR,
    A4CR_ADDR, A4DR_ADDR,
    A5CR_ADDR, A5DR_ADDR,
};

use AdcState::*;
use AdcPin::*;

mod states {
    use super::*;

    // TODO: Clean this up!

/*    single_test! {
        Adc_cr_pin0_read_output,
        pre: |p| { Adc::set_state(p, A0, Output).unwrap(); },
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { R0: 0b01 },
        memory: { }
    }

    single_test! {
        Adc_cr_pin0_set_output_valid,
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b01 },
        memory: { },
        post: |i| { eq!(Output, Adc::get_state(i.get_peripherals(), A0)); }
    }

    single_test! {
        Adc_cr_pin0_set_output_invalid,
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b1101 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b1101 },
        memory: { },
        post: |i| { eq!(Output, Adc::get_state(i.get_peripherals(), A0)); }
    }

}

mod read {
    use super::*;
    use lc3_traits::peripherals::Adc::*;

    single_test! {
        Adc_cr_pin0_read_input1,
        pre: |i| { Adc::set_state(i, A0, Input).unwrap();},{Adc::write(i, A0, #0b00001101).unwrap();},
        prefill: { 0x3010: A0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { Adc::read(i.get_peripherals(), A0); }
    }

    single_test! {
        Adc_cr_pin2_read_input,
        pre: |i| { Adc::set_state(i, A2, Input).unwrap(); },
        prefill: { 0x3010: A2DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01111111 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b01111111},
        memory: { },
        post: |i| { assert_eq!(0b01111111, Adc::read(i.get_peripherals(), A2)); }
    }
    single_test! {
        Adc_cr_pin3_read_input,
        pre: |i| { Adc::set_state(i, A3, Input).unwrap(); },
        prefill: { 0x3010: A3DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000000 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000000},
        memory: { },
        post: |i| { assert_eq!(0b00000000, Adc::read(i.get_peripherals(), A3)); }
    }
    single_test! {
        Adc_cr_pin4_read_input,
        pre: |i| { Adc::set_state(i, A4, Input).unwrap(); },
        prefill: { 0x3010: A4DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A4)); }
    }
    single_test! {
        Adc_cr_pin1_read_input,
        pre: |i| { Adc::set_state(i, A1, Input).unwrap(); },
        prefill: { 0x3010: A1DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A1)); }
    }
    single_test! {
        Adc_cr_pin5_read_input,
        pre: |i| { Adc::set_state(i, A5, Input).unwrap(); },
        prefill: { 0x3010: A5DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A5)); }
    }
    single_test! {
        Adc_cr_pin6_read_input,
        pre: |i| { Adc::set_state(i, A6, Input).unwrap(); },
        prefill: { 0x3010: A6DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00101111},
        memory: { },
        post: |i| { assert_eq!(0b00101111, Adc::read(i.get_peripherals(), A6)); }
    }
    single_test! {
        Adc_cr_pin7_read_input,
        pre: |i| { Adc::set_state(i, A7, Input).unwrap(); },
        prefill: { 0x3010: A7DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000001 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000001},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A7)); }
    }
    single_test! {
        Adc_cr_pin0_read_input1,
        pre: |i| { Adc::set_state(i, A0, Input).unwrap(); },
        prefill: {0x3010: A0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A0)); }
    }

    single_test! {
        Adc_cr_pin2_read_input,
        pre: |i| { Adc::set_state(i, A2, Input).unwrap(); },
        prefill: { 0x3010: A2DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01111111 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b01111111},
        memory: { },
        post: |i| { assert_eq!(0b01111111, Adc::read(i.get_peripherals(), A2)); }
    }
    single_test! {
        Adc_cr_pin3_read_input,
        pre: |i| { Adc::set_state(i, A3, Input).unwrap(); },
        prefill: { 0x3010: A3DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000000 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000000},
        memory: { },
        post: |i| { assert_eq!(0b00000000, Adc::read(i.get_peripherals(), A3)); }
    }
    single_test! {
        Adc_cr_pin4_read_input,
        pre: |i| { Adc::set_state(i, A4, Input).unwrap(); },
        prefill: { 0x3010: A4DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A4)); }
    }
    single_test! {
        Adc_cr_pin1_read_input,
        pre: |i| { Adc::set_state(i, A1, Input).unwrap(); },
        prefill: { 0x3010: A1DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A1)); }
    }
    single_test! {
        Adc_cr_pin5_read_input,
        pre: |i| { Adc::set_state(i, A5, Input).unwrap(); },
        prefill: { 0x3010: A5DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Adc::read(i.get_peripherals(), A5)); }
    }*/
}

mod write {
    use super::*;
    use lc3_traits::peripherals::Adc::*;

    // TODO: Clean this up!

    /*single_test! {
        Adc_cr_pin0_write_input1,
        pre: |i| { Adc::set_state(i, A0, Output).unwrap(); }, {Adc::write(i, A0, #0b00001101).unwrap();},
        prefill: {0x3010: A0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {A0DR_ADDR, #0b00001101},
    }

    single_test! {
        Adc_cr_pin1_write_input1,
        pre: |i| { Adc::set_state(i, A1, Output).unwrap(); }, {Adc::write(i, A1, #0b00000001).unwrap();},
        prefill: {0x3010: A1DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000001 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000001},
        memory: {A1DR_ADDR, #0b00000001}
    }
    single_test! {
        Adc_cr_pin2_write_input1,
        pre: |i| { Adc::set_state(i, A2, Output).unwrap(); }, {Adc::write(i, A2, #0b00001101).unwrap();},
        prefill: {0x3010: A2DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {A2DR_ADDR, #0b00001101}
    }
    single_test! {
        Adc_cr_pin3_write_input1,
        pre: |i| { Adc::set_state(i, A3, Output).unwrap(); }, {Adc::write(i, A3, #0b00001101).unwrap();},
        prefill: {0x3010: A3DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {A3DR_ADDR, #0b00001101}
    }
    single_test! {
        Adc_cr_pin4_write_input1,
        pre: |i| { Adc::set_state(i, A4, Output).unwrap(); }, {Adc::write(i, A4, #0b00001101).unwrap();},
        prefill: {0x3010: A4DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {A4DR_ADDR, #0b00001101}
    }
    single_test! {
        Adc_cr_pin5_write_input1,
        pre: |i| { Adc::set_state(i, A5, Output).unwrap(); }, {Adc::write(i, A5, #0b00001101).unwrap();},
        prefill: {0x3010: A5DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {A5DR_ADDR, #0b00001101}
    }*/
}
