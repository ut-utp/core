use super::*;

use lc3_traits::peripherals::adc::{Adc, AdcPin, AdcState, ADC_PINS, AdcReadError};
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
    #[test]
    fn exhaustive_state_testing() { with_larger_stack(None, || {
        assert_eq!(AdcPin::NUM_PINS, 6, "Number of Adc Pins has changed!");

        fn state_to_word(s: AdcState) -> SignedWord {
            match s {
                Disabled => 0b00,
                Enabled => 0b01,
            }
        }

        let state_iter = [Disabled, Enabled].iter();

        let permutations = ADC_PINS.iter()
            .map(|_| state_iter.clone())
            .multi_cartesian_product();

        for states in permutations {
            // Read test:
            single_test_inner! {
                prefill: {
                    0x3010: A0CR_ADDR,
                    0x3011: A1CR_ADDR,
                    0x3012: A2CR_ADDR,
                    0x3013: A3CR_ADDR,
                    0x3014: A4CR_ADDR,
                    0x3015: A5CR_ADDR,
                },
                insns: [
                    { LDI R0, #0xF }, // A0
                    { LDI R1, #0xF }, // A1
                    { LDI R2, #0xF }, // A2
                    { LDI R3, #0xF }, // A3
                    { LDI R4, #0xF }, // A4
                    { LDI R5, #0xF }, // A5
                ],
                steps: AdcPin::NUM_PINS,
                regs: {
                    R0: state_to_word(*states[0]) as Word,
                    R1: state_to_word(*states[1]) as Word,
                    R2: state_to_word(*states[2]) as Word,
                    R3: state_to_word(*states[3]) as Word,
                    R4: state_to_word(*states[4]) as Word,
                    R5: state_to_word(*states[5]) as Word,
                },
                pre: |p| {
                    for (pin, state) in ADC_PINS.iter().zip(states.clone()) {
                        Adc::set_state(p, *pin, *state).unwrap();
                    }
                },
            }

            // Write test:
            single_test_inner! {
                prefill_expr: {
                    (0x3020 + (0 * 3) + 2) /*0x3022*/: A0CR_ADDR,
                    (0x3020 + (1 * 3) + 2) /*0x3025*/: A1CR_ADDR,
                    (0x3020 + (2 * 3) + 2) /*0x3028*/: A2CR_ADDR,
                    (0x3020 + (3 * 3) + 2) /*0x302B*/: A3CR_ADDR,
                    (0x3020 + (4 * 3) + 2) /*0x302E*/: A4CR_ADDR,
                    (0x3020 + (5 * 3) + 2) /*0x3031*/: A5CR_ADDR,
                },
                insns: [
                    { AND R0, R0, #0 },
                    { ADD R0, R0, #(state_to_word(*states[0])) },
                    { STI R0, #0x1F }, // A0

                    { AND R1, R1, #0 },
                    { ADD R1, R1, #(state_to_word(*states[1])) },
                    { STI R1, #0x1F }, // A1

                    { AND R2, R2, #0 },
                    { ADD R2, R2, #(state_to_word(*states[2])) },
                    { STI R2, #0x1F }, // A2

                    { AND R3, R3, #0 },
                    { ADD R3, R3, #(state_to_word(*states[3])) },
                    { STI R3, #0x1F }, // A3

                    { AND R4, R4, #0 },
                    { ADD R4, R4, #(state_to_word(*states[4])) },
                    { STI R4, #0x1F }, // A4

                    { AND R5, R5, #0 },
                    { ADD R5, R5, #(state_to_word(*states[5])) },
                    { STI R5, #0x1F }, // A5

                ],
                steps: AdcPin::NUM_PINS * 3,
                post: |i| {
                    for (pin, state) in ADC_PINS.iter().zip(states.clone()) {
                        let got = Adc::get_state(i.get_peripherals(), *pin);
                        eq!(
                            *state, got,
                            "Adc Pin {}: expected state `{:?}`, got `{:?}`.\
                            \n\n[Test Case: {:?}]",
                            pin, *state, got, states
                        );
                    }
                }
            }
        }
    })}


    single_test! {
        adc_cr_pin0_disabled_read,
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { R0: 0b00 },
        pre: |p| { Adc::set_state(p, A0, Disabled).unwrap(); },
    }

    single_test! {
        adc_cr_pin0_set_disabled_write,
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b00 },
        post: |i| { eq!(Disabled, Adc::get_state(i.get_peripherals(), A0)); }
    }

    single_test! {
        adc_cr_pin0_enabled_read,
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { R0: 0b01 },
        pre: |p| { Adc::set_state(p, A0, Enabled).unwrap(); },
    }

    single_test! {
        adc_cr_pin0_set_enabled_write,
        prefill: { 0x3010: A0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b01 },
        post: |i| { eq!(Enabled, Adc::get_state(i.get_peripherals(), A0)); }
    }

}

mod read {
    use super::*;

    #[test]
    fn read_testing() { with_larger_stack(None, || {
         assert_eq!(AdcPin::NUM_PINS, 6, "Number of Adc Pins has changed!");

      
         let state_iter = [Disabled, Enabled].iter();


        fn match_state(s: AdcState) -> u16 {
            match s {
                Disabled => {
                    32768
                },
                Enabled => {
                    0
                }


            }
        }

         let permutations = ADC_PINS.iter()
             .map(|_| state_iter.clone())
             .multi_cartesian_product();
 
        let mut counter = 0; // easier than starting an rng thread... 
         for states in permutations {

           


            single_test_inner! {
                prefill: {
                    0x3010: A0DR_ADDR,
                    0x3011: A1DR_ADDR,
                    0x3012: A2DR_ADDR,
                    0x3013: A3DR_ADDR,
                    0x3014: A4DR_ADDR,
                    0x3015: A5DR_ADDR,
                },
                insns: [
                    { LDI R0, #0xF }, // A0
                    { LDI R1, #0xF }, // A1
                    { LDI R2, #0xF }, // A2
                    { LDI R3, #0xF }, // A3
                    { LDI R4, #0xF }, // A4
                    { LDI R5, #0xF }, // A5
                ],
                steps: AdcPin::NUM_PINS,
                regs: {
                    R0: match_state(*states[0]),
                    R1: match_state(*states[1]),
                    R2: match_state(*states[2]),
                    R3: match_state(*states[3]),
                    R4: match_state(*states[4]),
                    R5: match_state(*states[5]),
                },
                pre: |p| { 
                    for (pin, state) in ADC_PINS.iter().zip(states.clone()) {
                        let _set = Adc::set_state(p, *pin, *state).unwrap(); 
                        // let _set_val = Adc::set_value(p, *pin, counter); - no implementation for RwLock
                    }
                },   
            }


        }   


    })}
    
    /*
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
//}

mod write {
    use super::*;


    #[test]
    fn write_testing() { with_larger_stack(None, || {
         assert_eq!(AdcPin::NUM_PINS, 6, "Number of Adc Pins has changed!");

      
         let state_iter = [Disabled, Enabled].iter();

         let permutations = ADC_PINS.iter()
             .map(|_| state_iter.clone())
             .multi_cartesian_product();
 

        fn state_to_word(s: AdcState) -> SignedWord {
                match s {
                    Disabled => 0b00,
                    Enabled => 0b01,
                }
            }

         for states in permutations {

          // Write test:
          single_test_inner! {
            prefill_expr: {
                (0x3020 + (0 * 3) + 2) /*0x3022*/: A0CR_ADDR,
                (0x3020 + (1 * 3) + 2) /*0x3025*/: A1CR_ADDR,
                (0x3020 + (2 * 3) + 2) /*0x3028*/: A2CR_ADDR,
                (0x3020 + (3 * 3) + 2) /*0x302B*/: A3CR_ADDR,
                (0x3020 + (4 * 3) + 2) /*0x302E*/: A4CR_ADDR,
                (0x3020 + (5 * 3) + 2) /*0x3031*/: A5CR_ADDR,
            },
            insns: [
                { AND R0, R0, #0 },
                { ADD R0, R0, #(state_to_word(*states[0])) },
                { STI R0, #0x1F }, // A0

                { AND R1, R1, #0 },
                { ADD R1, R1, #(state_to_word(*states[1])) },
                { STI R1, #0x1F }, // A1

                { AND R2, R2, #0 },
                { ADD R2, R2, #(state_to_word(*states[2])) },
                { STI R2, #0x1F }, // A2

                { AND R3, R3, #0 },
                { ADD R3, R3, #(state_to_word(*states[3])) },
                { STI R3, #0x1F }, // A3

                { AND R4, R4, #0 },
                { ADD R4, R4, #(state_to_word(*states[4])) },
                { STI R4, #0x1F }, // A4

                { AND R5, R5, #0 },
                { ADD R5, R5, #(state_to_word(*states[5])) },
                { STI R5, #0x1F }, // A5

            ],
            steps: AdcPin::NUM_PINS * 3,
            post: |i| {
                for (pin, state) in ADC_PINS.iter().zip(states.clone()) {
                    let got = Adc::get_state(i.get_peripherals(), *pin);
                    let read_adc = Adc::read(i.get_peripherals(), *pin);

                    match got {
                        Enabled => {
                            eq!(read_adc, Ok(0), 
                            "Adc Pin {}: expected val `{:?}`, got `{:?}`.\
                            \n\n[Test Case: {:?}]",
                            pin, 0, read_adc, states
                            );
                        },
                        Disabled => {
                            eq!(read_adc, Err(AdcReadError((*pin, Disabled))),
                            "Adc Pin {}: expected Err, got `{:?}`.\
                            \n\n[Test Case: {:?}]",
                            pin, read_adc, states
                        );
                        }
                    }

                   
                }
            }
        }
    }
    })}
}



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

