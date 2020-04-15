use super::*;

use lc3_traits::peripherals::gpio::{Gpio, GpioPin, GpioState, GPIO_PINS};
use lc3_baseline_sim::interp::InstructionInterpreter;
use lc3_baseline_sim::mem_mapped::{
    G0CR_ADDR, G0DR_ADDR, G0_INT_VEC,
    G1CR_ADDR, G1DR_ADDR, G1_INT_VEC,
    G2CR_ADDR, G2DR_ADDR, G2_INT_VEC,
    G3CR_ADDR, G3DR_ADDR, G3_INT_VEC,
    G4CR_ADDR, G4DR_ADDR, G4_INT_VEC,
    G5CR_ADDR, G5DR_ADDR, G5_INT_VEC,
    G6CR_ADDR, G6DR_ADDR, G6_INT_VEC,
    G7CR_ADDR, G7DR_ADDR, G7_INT_VEC,
    GPIODR_ADDR,
};

use GpioState::*;
use GpioPin::*;

mod states {
    use super::*;

    // The idea is to test all the valid configurations ((4 states) ^
    // GpioPin::NUM_PINS -> 65536) for the set of Gpio Pins.
    //
    // And then to individually test some specific edge cases (everything but
    // the first function below).

    #[test]
    fn exhaustive_state_testing() { with_larger_stack(None, || {
        // The actual assembly assumes that there are 8 pins and needs to be
        // updated if this changes.
        assert_eq!(GpioPin::NUM_PINS, 8, "Number of Gpio Pins has changed!");

        // We're also assuming that the states in GpioState won't change:
        // TODO: make this a `From` impl on the GpioState type so we don't have
        // to remember/copy this around.
        fn state_to_word(s: GpioState) -> SignedWord {
            match s {
                Disabled => 0b00,
                Output => 0b01,
                Input => 0b10,
                Interrupt => 0b11,
            }
        }

        let state_iter = [Disabled, Output, Input, Interrupt].iter();

        let permutations = GPIO_PINS.iter()
            .map(|_| state_iter.clone())
            .multi_cartesian_product();

        for states in permutations {
            // Read test:
            single_test_inner! {
                pre: |p| {
                    for (pin, state) in GPIO_PINS.iter().zip(states.clone()) {
                        Gpio::set_state(p, *pin, *state).unwrap();
                    }
                },
                prefill: {
                    0x3010: G0CR_ADDR,
                    0x3011: G1CR_ADDR,
                    0x3012: G2CR_ADDR,
                    0x3013: G3CR_ADDR,
                    0x3014: G4CR_ADDR,
                    0x3015: G5CR_ADDR,
                    0x3016: G6CR_ADDR,
                    0x3017: G7CR_ADDR
                },
                insns: [
                    { LDI R0, #0xF }, // G0
                    { LDI R1, #0xF }, // G1
                    { LDI R2, #0xF }, // G2
                    { LDI R3, #0xF }, // G3
                    { LDI R4, #0xF }, // G4
                    { LDI R5, #0xF }, // G5
                    { LDI R6, #0xF }, // G6
                    { LDI R7, #0xF }, // G7
                ],
                steps: GpioPin::NUM_PINS,
                regs: {
                    R0: state_to_word(*states[0]) as Word,
                    R1: state_to_word(*states[1]) as Word,
                    R2: state_to_word(*states[2]) as Word,
                    R3: state_to_word(*states[3]) as Word,
                    R4: state_to_word(*states[4]) as Word,
                    R5: state_to_word(*states[5]) as Word,
                    R6: state_to_word(*states[6]) as Word,
                    R7: state_to_word(*states[7]) as Word,
                },
                memory: { }
            }

            // Write test:
            single_test_inner! {
                prefill: {
                    /*(0x3020 + (0 * 3) + 2)*/ 0x3022: G0CR_ADDR,
                    /*(0x3020 + (1 * 3) + 2)*/ 0x3025: G1CR_ADDR,
                    /*(0x3020 + (2 * 3) + 2)*/ 0x3028: G2CR_ADDR,
                    /*(0x3020 + (3 * 3) + 2)*/ 0x302B: G3CR_ADDR,
                    /*(0x3020 + (4 * 3) + 2)*/ 0x302E: G4CR_ADDR,
                    /*(0x3020 + (5 * 3) + 2)*/ 0x3031: G5CR_ADDR,
                    /*(0x3020 + (6 * 3) + 2)*/ 0x3034: G6CR_ADDR,
                    /*(0x3020 + (7 * 3) + 2)*/ 0x3037: G7CR_ADDR
                },
                insns: [
                    { AND R0, R0, #0 },
                    { ADD R0, R0, #(state_to_word(*states[0])) },
                    { STI R0, #0x1F }, // G0

                    { AND R1, R1, #0 },
                    { ADD R1, R1, #(state_to_word(*states[1])) },
                    { STI R1, #0x1F }, // G1

                    { AND R2, R2, #0 },
                    { ADD R2, R2, #(state_to_word(*states[2])) },
                    { STI R2, #0x1F }, // G2

                    { AND R3, R3, #0 },
                    { ADD R3, R3, #(state_to_word(*states[3])) },
                    { STI R3, #0x1F }, // G3

                    { AND R4, R4, #0 },
                    { ADD R4, R4, #(state_to_word(*states[4])) },
                    { STI R4, #0x1F }, // G4

                    { AND R5, R5, #0 },
                    { ADD R5, R5, #(state_to_word(*states[5])) },
                    { STI R5, #0x1F }, // G5

                    { AND R6, R6, #0 },
                    { ADD R6, R6, #(state_to_word(*states[6])) },
                    { STI R6, #0x1F }, // G6

                    { AND R7, R7, #0 },
                    { ADD R7, R7, #(state_to_word(*states[7])) },
                    { STI R7, #0x1F }, // G7
                ],
                steps: GpioPin::NUM_PINS * 3,
                regs: { },
                memory: { },
                post: |i| {
                    for (pin, state) in GPIO_PINS.iter().zip(states.clone()) {
                        let got = Gpio::get_state(i.get_peripherals(), *pin);
                        eq!(
                            *state, got,
                            "Gpio Pin {}: expected state `{:?}`, got `{:?}`.\
                            \n\n[Test Case: {:?}]",
                            pin, *state, got, states
                        );
                    }
                }
            }
        }
    })}

    // Just to make sure nothing _relies_ on us setting/reading from all the
    // pins, we do a couple of hardcoded tests:
    single_test! {
        gpio_cr_pin0_read_output,
        pre: |p| { Gpio::set_state(p, G0, Output).unwrap(); },
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { R0: 0b01 },
        memory: { }
    }

    single_test! {
        gpio_cr_pin0_set_output_valid,
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b01 },
        memory: { },
        post: |i| { eq!(Output, Gpio::get_state(i.get_peripherals(), G0)); }
    }

    // We should also test that we actually only use the lower two bits of the
    // value we're given in the G*_CR registers.
    single_test! {
        gpio_cr_pin0_set_output_invalid,
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b1101 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b1101 },
        memory: { },
        post: |i| { eq!(Output, Gpio::get_state(i.get_peripherals(), G0)); }
    }

}

mod read {
    use super::*;
    use lc3_traits::peripherals::gpio::*;
    // Test that reads of [0, 1] work for all the pins when everything is
    // configured as inputs (i think we can skip the full 2 ^ 8 possibilities
    // this time)

    // Test that each pin works (0 and 1) as an input individually
    single_test! {
        gpio_cr_pin0_read_input1,
        pre: |i| { Gpio::set_state(i, G0, Input).unwrap();},{Gpio::write(i, G0, #0b00001101).unwrap();},
        prefill: { 0x3010: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { Gpio::read(i.get_peripherals(), G0); }
    }

    single_test! {
        gpio_cr_pin2_read_input,
        pre: |i| { Gpio::set_state(i, G2, Input).unwrap(); },
        prefill: { 0x3010: G2DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01111111 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b01111111},
        memory: { },
        post: |i| { assert_eq!(0b01111111, Gpio::read(i.get_peripherals(), G2)); }
    }
    single_test! {
        gpio_cr_pin3_read_input,
        pre: |i| { Gpio::set_state(i, G3, Input).unwrap(); },
        prefill: { 0x3010: G3DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000000 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000000},
        memory: { },
        post: |i| { assert_eq!(0b00000000, Gpio::read(i.get_peripherals(), G3)); }
    }
    single_test! {
        gpio_cr_pin4_read_input,
        pre: |i| { Gpio::set_state(i, G4, Input).unwrap(); },
        prefill: { 0x3010: G4DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G4)); }
    }
    single_test! {
        gpio_cr_pin1_read_input,
        pre: |i| { Gpio::set_state(i, G1, Input).unwrap(); },
        prefill: { 0x3010: G1DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G1)); }
    }
    single_test! {
        gpio_cr_pin5_read_input,
        pre: |i| { Gpio::set_state(i, G5, Input).unwrap(); },
        prefill: { 0x3010: G5DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G5)); }
    }
    single_test! {
        gpio_cr_pin6_read_input,
        pre: |i| { Gpio::set_state(i, G6, Input).unwrap(); },
        prefill: { 0x3010: G6DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00101111},
        memory: { },
        post: |i| { assert_eq!(0b00101111, Gpio::read(i.get_peripherals(), G6)); }
    }
    single_test! {
        gpio_cr_pin7_read_input,
        pre: |i| { Gpio::set_state(i, G7, Input).unwrap(); },
        prefill: { 0x3010: G7DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000001 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000001},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G7)); }
    }
    // Test that reads when in interrupt mode work (0 and 1; be sure to set
    // the value and then switch to interrupt mode so you don't trigger an
    // interrupt); test this on all pins
    single_test! {
        gpio_cr_pin0_read_input1,
        pre: |i| { Gpio::set_state(i, G0, Input).unwrap(); },
        prefill: {0x3010: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G0)); }
    }

    single_test! {
        gpio_cr_pin2_read_input,
        pre: |i| { Gpio::set_state(i, G2, Input).unwrap(); },
        prefill: { 0x3010: G2DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01111111 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b01111111},
        memory: { },
        post: |i| { assert_eq!(0b01111111, Gpio::read(i.get_peripherals(), G2)); }
    }
    single_test! {
        gpio_cr_pin3_read_input,
        pre: |i| { Gpio::set_state(i, G3, Input).unwrap(); },
        prefill: { 0x3010: G3DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000000 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000000},
        memory: { },
        post: |i| { assert_eq!(0b00000000, Gpio::read(i.get_peripherals(), G3)); }
    }
    single_test! {
        gpio_cr_pin4_read_input,
        pre: |i| { Gpio::set_state(i, G4, Input).unwrap(); },
        prefill: { 0x3010: G4DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G4)); }
    }
    single_test! {
        gpio_cr_pin1_read_input,
        pre: |i| { Gpio::set_state(i, G1, Input).unwrap(); },
        prefill: { 0x3010: G1DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G1)); }
    }
    single_test! {
        gpio_cr_pin5_read_input,
        pre: |i| { Gpio::set_state(i, G5, Input).unwrap(); },
        prefill: { 0x3010: G5DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000101},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G5)); }
    }
    single_test! {
        gpio_cr_pin6_read_input,
        pre: |i| { Gpio::set_state(i, G6, Input).unwrap(); },
        prefill: { 0x3010: G6DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00101111},
        memory: { },
        post: |i| { assert_eq!(0b00101111, Gpio::read(i.get_peripherals(), G6)); }
    }
    single_test! {
        gpio_cr_pin7_read_input,
        pre: |i| { Gpio::set_state(i, G7, Input).unwrap(); },
        prefill: { 0x3010: G7DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000001 },{STI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000001},
        memory: { },
        post: |i| { assert_eq!(0b00001101, Gpio::read(i.get_peripherals(), G7)); }
    }
    // Test that reads when in output mode work (i.e. they set the high bit)
    // be sure to also test this with more than 1 pin in output mode

    // If you're feeling ambitious:
    // There are 7 states each pin can be in:
    //   - Input(false), Input(true),
    //   - Output(false), Output(true),
    //   - Interrupt(false), Interrupt(true),
    //   - Disabled
    //
    // Test reads on all 8 pins for all the possible combinations of states;
    // this comes out to (7 ^ 8) states: 5,764,801 states. This is _exhaustive_,
    // but as with the mode testing, we should leave the other tests in anyways
    // in case some bad impl relies on the sequence of 8 reads.

    // Test the whole port register when all the pins are inputs with some
    // values (again: don't need to do all 256)

    // Test the whole port register when *not* all the pins are inputs. Should
    // test with at least 1 kind of each other state (i.e. at least one pin is
    // disabled, at least one is in output mode, etc.)
    // Should also test that reads for the whole port do the right thing when
    // *some* of the pins are in interrupt mode
}

mod write {
    use super::*;
    use lc3_traits::peripherals::gpio::*;
        single_test! {
        gpio_cr_pin0_write_input1,
        pre: |i| { Gpio::set_state(i, G0, Output).unwrap(); }, {Gpio::write(i, G0, #0b00001101).unwrap();},
        prefill: {0x3010: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G0DR_ADDR, #0b00001101},
    }

    single_test! {
        gpio_cr_pin1_write_input1,
        pre: |i| { Gpio::set_state(i, G1, Output).unwrap(); }, {Gpio::write(i, G1, #0b00000001).unwrap();},
        prefill: {0x3010: G1DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00000001 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00000001},
        memory: {G1DR_ADDR, #0b00000001}
    }
    single_test! {
        gpio_cr_pin2_write_input1,
        pre: |i| { Gpio::set_state(i, G2, Output).unwrap(); }, {Gpio::write(i, G2, #0b00001101).unwrap();},
        prefill: {0x3010: G2DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G2DR_ADDR, #0b00001101}
    }
    single_test! {
        gpio_cr_pin3_write_input1,
        pre: |i| { Gpio::set_state(i, G3, Output).unwrap(); }, {Gpio::write(i, G3, #0b00001101).unwrap();},
        prefill: {0x3010: G3DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G3DR_ADDR, #0b00001101}
    }
    single_test! {
        gpio_cr_pin4_write_input1,
        pre: |i| { Gpio::set_state(i, G4, Output).unwrap(); }, {Gpio::write(i, G4, #0b00001101).unwrap();},
        prefill: {0x3010: G4DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G4DR_ADDR, #0b00001101}
    }
    single_test! {
        gpio_cr_pin5_write_input1,
        pre: |i| { Gpio::set_state(i, G5, Output).unwrap(); }, {Gpio::write(i, G5, #0b00001101).unwrap();},
        prefill: {0x3010: G5DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G5DR_ADDR, #0b00001101}
    }
    single_test! {
        gpio_cr_pin6_write_input1,
        pre: |i| { Gpio::set_state(i, G6, Output).unwrap(); }, {Gpio::write(i, G6, #0b00001101).unwrap();},
        prefill: {0x3010: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G6DR_ADDR, #0b00001101}
    }
    single_test! {
        gpio_cr_pin7_write_input1,
        pre: |i| { Gpio::set_state(i, G7, Output).unwrap(); }, {Gpio::write(i, G7, #0b00001101).unwrap();},
        prefill: {0x3010: G7DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b00001101 },{LDI R0, #0xD }],
        steps: 3,
        regs: { R0: 0b00001101},
        memory: {G7DR_ADDR, #0b00001101},
    }
}

//mod interrupt {
//    // Reading from pins in interrupt mode should already be covered; the only
//    // thing left is to test that interrupts actually trigger.
//
//    // Here are the variables:
//    //   - rising edge or falling edge
//    //   - in interrupt mode or some other mode (i.e. 3 other modes)
//
//    // Interrupts should only trigger on rising edges AND when interrupts are
//    // enabled AND when in interrupt mode. If we do an exhaustive test, this
//    // is (2 * 4) ^ 8 = 16,777,216 states...
//    //
//    // So, maybe don't do an exhaustive test or randomly pick a few thousand
//    // combinations from the full set of possibilities.
//
//    // Should also test that multiple interrupts are handled (i.e. they all
//    // run).
//
//    // Also need to test that when multiple interrupts occur, they trigger in
//    // the documented order!
//    //
//    // i.e. if G0 through G7 all trigger, G0 runs first, then G1, then G2, etc.
//    //
//    // One way we can actually test this is to have each handler increment R0
//    // and to have each handler store R0 into a fixed memory location for that
//    // handler.
//    //
//    // i.e. G0's handler -> 0x1000
//    //      G1's handler -> 0x1001
//    //      G2's handler -> 0x1002
//    //      G3's handler -> 0x1003
//    //      etc.
//    //
//    // If the handlers trigger in the right order, the values in 0x1000..0x1007
//    // should be sequential; if the handlers get run out of order they won't be.
//}

mod errors {
    use super::*;
    use lc3_traits::peripherals::gpio::{GpioWriteError, GpioReadError};
    use lc3_traits::error::Error;

    single_test! {
        gpio_write_error_disabled,
        prefill: { 0x3010: G0DR_ADDR },
        insns: [ { STI R0, #0xF } ],
        steps: 1,
        regs: { },
        memory: { },
        post: |i| { eq!(Error::from(GpioWriteError((G0, Disabled))), InstructionInterpreter::get_error(i).unwrap()); }
    }

    single_test! {
        gpio_write_error_input,
        prefill: { 0x3010: G0CR_ADDR, 0x3011: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b10 }, { STI R0, #0xD }, { STI R0, #0xD } ],
        steps: 4,
        regs: { R0: 0b10 },
        memory: { },
        post: |i| { eq!(Error::from(GpioWriteError((G0, Input))), InstructionInterpreter::get_error(i).unwrap()); }
    }

    single_test! {
        gpio_read_error_disabled,
        prefill: { 0x3010: G0DR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { },
        memory: { },
        post: |i| { eq!(Error::from(GpioReadError((G0, Disabled))), InstructionInterpreter::get_error(i).unwrap()); }
    }

    single_test! {
        gpio_read_error_output,
        prefill: { 0x3010: G0CR_ADDR, 0x3011: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD }, { LDI R0, #0xD } ],
        steps: 4,
        regs: { R0: 0x8000 },
        memory: { },
        post: |i| { eq!(Error::from(GpioReadError((G0, Output))), InstructionInterpreter::get_error(i).unwrap()); }
    }
}
