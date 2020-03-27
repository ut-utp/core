use lc3_isa::{insn, Addr, Instruction, Reg, Word, SignedWord};

use lc3_shims::memory::MemoryShim;
use lc3_shims::peripherals::PeripheralsShim;

use pretty_assertions::assert_eq as eq;

#[path = "common.rs"]
mod common;
use common::interp_test_runner;

#[macro_use]
extern crate itertools;
use itertools::Itertools;

// Setup func runs before anything is set; teardown func runs after everything is
// checked but the order shouldn't matter.
macro_rules! single_test {
    ($(|$panics:literal|)?
        $name:ident,
        $(pre: |$peripherals_s:ident| $setup:block,)?
        $(prefill: { $($addr_p:literal: $val_p:expr),* $(,)?},)?
        insns: [ $({ $($insn:tt)* }),* $(,)?],
        $(steps: $steps:expr,)?
        regs: { $($r:tt: $v:expr),* $(,)?},
        memory: { $($addr:literal: $val:expr),* $(,)?} $(,)?
        $(post: |$peripherals_t:ident| $teardown:block)? $(,)?
    ) => {
    $(#[doc = $panics] #[should_panic])?
    #[test]
    fn $name() {
        single_test_inner!(
            $(pre: |$peripherals_s| $setup,)?
            $(prefill: { $($addr_p: $val_p),* },)?
            insns: [ $({ $($insn)* }),* ],
            $(steps: $steps,)?
            regs: { $($r: $v),* },
            memory: { $($addr: $val),* }
            $(post: |$peripherals_t| $teardown)?
        );
    }};
}

macro_rules! single_test_inner {
    (   $(pre: |$peripherals_s:ident| $setup:block,)?
        $(prefill: { $($addr_p:literal: $val_p:expr),* $(,)?},)?
        insns: [ $({ $($insn:tt)* }),* $(,)?],
        $(steps: $steps:expr,)?
        regs: { $($r:tt: $v:expr),* $(,)?},
        memory: { $($addr:literal: $val:expr),* $(,)?} $(,)?
        $(post: |$peripherals_t:ident| $teardown:block)? $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut regs: [Option<Word>; Reg::NUM_REGS] = [None, None, None, None, None, None, None, None];
        $(regs[Into::<u8>::into($r) as usize] = Some($v);)*

        #[allow(unused_mut)]
        let mut checks: Vec<(Addr, Word)> = Vec::new();
        $(checks.push(($addr, $val));)*

        #[allow(unused_mut)]
        let mut prefill: Vec<(Addr, Word)> = Vec::new();
        $($(prefill.push(($addr_p, $val_p));)*)?

        #[allow(unused_mut)]
        let mut insns: Vec<Instruction> = Vec::new();
        $(insns.push(insn!($($insn)*));)*

        #[allow(unused)]
        let setup_func = |_p: &mut PeripheralsShim| { }; // no-op if not specified
        $(let setup_func = |$peripherals_s: &mut PeripheralsShim| $setup;)?

        #[allow(unused)]
        let teardown_func = |_p: &PeripheralsShim| { }; // no-op if not specified
        $(let teardown_func = |$peripherals_t: &PeripheralsShim| $teardown;)?

        #[allow(unused)]
        let steps: Option<usize> = None;
        $(let steps: Option<usize> = Some($steps))?;

        interp_test_runner::<MemoryShim, PeripheralsShim, _, _>(
            prefill,
            insns,
            steps,
            regs,
            None,
            checks,
            // (|_|{}),
            // (|_|{}),
            setup_func,
            teardown_func,
        );
    }};
}

#[cfg(test)]
mod gpio_mem_mapped {
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
        // And then to individually test some specific edge cases (everything
        // but the first function below).

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

            // fn word_to_state(s: Word) -> GpioState {
            //     match s {
            //         0b00 => Disabled,
            //         0b01 => Output,
            //         0b10 => Input,
            //         0b11 => Interrupt,
            //         _ => panic!("Got back an invalid state number {}.", s),
            //     }
            // }

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
