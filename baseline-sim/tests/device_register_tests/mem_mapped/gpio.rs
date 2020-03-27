use super::*;

use lc3_traits::peripherals::gpio::{Gpio, GpioPin, GpioState, GPIO_PINS};
use lc3_baseline_sim::mem_mapped::{G0CR_ADDR, G1CR_ADDR, G2CR_ADDR, G3CR_ADDR, G4CR_ADDR, G5CR_ADDR, G6CR_ADDR, G7CR_ADDR};

use Reg::*;
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
    fn exhaustive_state_testing() {
        // The actual assembly assumes that there are 8 pins and needs to be
        // updated if this changes.
        assert_eq!(GpioPin::NUM_PINS, 8, "Number of Gpio Pins has changed!");

        // We're also assuming that the states in GpioState won't change:
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
                post: |p| {
                    for (pin, state) in GPIO_PINS.iter().zip(states.clone()) {
                        let got = Gpio::get_state(p, *pin);
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
    }

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
        post: |p| { eq!(Output, Gpio::get_state(p, G0)); }
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
        post: |p| { eq!(Output, Gpio::get_state(p, G0)); }
    }
}

mod read {
    // Test that reads of [0, 1] work for all the pins when everything is
    // configured as inputs (i think we can skip the full 2 ^ 8 possibilities
    // this time)

    // Test that each pin works (0 and 1) as an input individually

    // Test that reads when in interrupt mode work (0 and 1; be sure to set
    // the value and then switch to interrupt mode so you don't trigger an
    // interrupt); test this on all pins

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
    // Literally the same as the read tests except you get different behavior
    // if you try to write to a pin that's in interrupt mode (as in: it
    // shouldn't work).
}

mod interrupt {
    // Reading from pins in interrupt mode should already be covered; the only
    // thing left is to test that interrupts actually trigger.

    // Here are the variables:
    //   - rising edge or falling edge
    //   - interrupts enabled or disabled
    //   - in interrupt mode or some other mode (i.e. 3 other modes)

    // Interrupts should only trigger on rising edges AND when interrupts are
    // enabled AND when in interrupt mode. If we do an exhaustive test, this
    // is (2 * 2 * 4) ^ 8 = 4,294,967,296 states...
    //
    // So, maybe don't do an exhaustive test or randomly pick a few thousand
    // combinations from the full set of possibilities.

    // Should also test that multiple interrupts are handled (i.e. they all
    // run).

    // Also need to test that when multiple interrupts occur, they trigger in
    // the documented order!
    //
    // i.e. if G0 through G7 all trigger, G0 runs first, then G1, then G2, etc.
    //
    // One way we can actually test this is to have each handler increment R0
    // and to have each handler store R0 into a fixed memory location for that
    // handler.
    //
    // i.e. G0's handler -> 0x1000
    //      G1's handler -> 0x1001
    //      G2's handler -> 0x1002
    //      G3's handler -> 0x1003
    //      etc.
    //
    // If the handlers trigger in the right order, the values in 0x1000..0x1007
    // should be sequential; if the handlers get run out of order they won't be.
}
