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
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;



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
                pre: |p| {
                    for (pin, state) in GPIO_PINS.iter().zip(states.clone()) {
                        Gpio::set_state(p, *pin, *state).unwrap();
                    }
                },
            }

            // Write test:
            single_test_inner! {
                prefill_expr: {
                    (0x3020 + (0 * 3) + 2) /*0x3022*/: G0CR_ADDR,
                    (0x3020 + (1 * 3) + 2) /*0x3025*/: G1CR_ADDR,
                    (0x3020 + (2 * 3) + 2) /*0x3028*/: G2CR_ADDR,
                    (0x3020 + (3 * 3) + 2) /*0x302B*/: G3CR_ADDR,
                    (0x3020 + (4 * 3) + 2) /*0x302E*/: G4CR_ADDR,
                    (0x3020 + (5 * 3) + 2) /*0x3031*/: G5CR_ADDR,
                    (0x3020 + (6 * 3) + 2) /*0x3034*/: G6CR_ADDR,
                    (0x3020 + (7 * 3) + 2) /*0x3037*/: G7CR_ADDR,
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
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { R0: 0b01 },
        pre: |p| { Gpio::set_state(p, G0, Output).unwrap(); },
    }

    single_test! {
        gpio_cr_pin0_set_output_valid,
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b01 },
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

    // TODO: clean this up!
    
    #[test]
    fn read_valid_states_test(){ with_larger_stack(None, || {

        let valid_state_iter = [Input, Interrupt].iter(); // test when set to input or output

        let permutations = GPIO_PINS.iter()
            .map(|_| valid_state_iter.clone())
            .multi_cartesian_product();

        for states in permutations {
            for iteration in 0..=255 { // test setting all pins to 1 or 0 -- all combinations

                let val = format!("{:08b}", iteration);
               
                let gpio_vals: Vec<char> =  val.chars().collect();
                let mut gpio_bools: Vec<u16> = Vec::<u16>::new();
                for values in gpio_vals.iter() {
                    gpio_bools.push((values.to_digit(2)).unwrap() as u16);
                }
    
                single_test_inner! {
                    prefill: {
                        0x3010: G0DR_ADDR,
                        0x3011: G1DR_ADDR,
                        0x3012: G2DR_ADDR,
                        0x3013: G3DR_ADDR,
                        0x3014: G4DR_ADDR,
                        0x3015: G5DR_ADDR,
                        0x3016: G6DR_ADDR,
                        0x3017: G7DR_ADDR,
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
                        R0: gpio_bools[0],
                        R1: gpio_bools[1],
                        R2: gpio_bools[2],
                        R3: gpio_bools[3],
                        R4: gpio_bools[4],
                        R5: gpio_bools[5],
                        R6: gpio_bools[6],
                        R7: gpio_bools[7],
                    },
                    pre: |p| { 
                        
                        for (num,  pin) in GPIO_PINS.iter().enumerate() { 
                            let _set = Gpio::set_state(p, *pin, Output).unwrap(); 
                            let state = Gpio::get_state(p, *pin);
                            eq!(state, Output);
                            
                            let pin_bool = gpio_bools[num] != 0; 
                            let write = Gpio::write(p, *pin, pin_bool);
                            eq!(write, Ok(()), "Write failure at pin {:?}, setting value to {}\nTest case: {:?}", *pin, pin_bool, gpio_vals);
    
    
                            let _set = Gpio::set_state(p, *pin, *states[num]).unwrap();
                            let state = Gpio::get_state(p, *pin);
                            eq!(state, Input);
                        }
                    }, 
                }
            }
        }
       
    })}



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



    #[test]
    fn read_output_testing() { with_larger_stack(None, || { 
        
        let state_iter = [Disabled, Output, Input, Interrupt].iter();
        

        let permutations = GPIO_PINS.iter()
            .map(|_| state_iter.clone())
            .multi_cartesian_product();

       
        fn match_state(s: GpioState) -> u16 {
            match s {
                Disabled | Output => {
                    32768
                },
                Input | Interrupt => {
                    0
                },

            }
        }

            
            for states in permutations { 
                

                single_test_inner! {
                    prefill: {
                        0x3010: G0DR_ADDR,
                        0x3011: G1DR_ADDR,
                        0x3012: G2DR_ADDR,
                        0x3013: G3DR_ADDR,
                        0x3014: G4DR_ADDR,
                        0x3015: G5DR_ADDR,
                        0x3016: G6DR_ADDR,
                        0x3017: G7DR_ADDR
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
                        R0: match_state(*states[0]),
                        R1: match_state(*states[1]),
                        R2: match_state(*states[2]),
                        R3: match_state(*states[3]),
                        R4: match_state(*states[4]),
                        R5: match_state(*states[5]),
                        R6: match_state(*states[6]),
                        R7: match_state(*states[7]),
                    },
                    pre: |p| {
                        for (pin, state) in GPIO_PINS.iter().zip(states.clone()) {
                           
                            let _set = Gpio::set_state(p, *pin, *state);
                            
                        }
                    },
                }

            }

    })}


}

mod write {
    use super::*;
    use lc3_traits::peripherals::gpio::*;



    
    // test that when you write in output mode that you can read the values back in 
    #[test]
    fn write_output_testing() { with_larger_stack(None, || {

        fn state_to_word(s: GpioState) -> SignedWord {
            match s {
                Disabled => 0b00,
                Output => 0b01,
                Input => 0b10,
                Interrupt => 0b11,
            }
        }
        
        
        for iteration in 0..=255 {

            let val = format!("{:08b}", iteration);
           
            let gpio_vals: Vec<char> =  val.chars().collect();
            let mut gpio_bools: Vec<u16> = Vec::<u16>::new();
            for values in gpio_vals.iter() {
                gpio_bools.push((values.to_digit(2)).unwrap() as u16);
            }

            // Write test:
            single_test_inner! {
                prefill_expr: {
                    (0x3050 + (0 * 6) + 2) /*0x3052*/: G0DR_ADDR,
                    (0x3050 + (0 * 6) + 3) /*0x3053*/: G0CR_ADDR,
                    (0x3050 + (1 * 6) + 2) /*0x3055*/: G1DR_ADDR,
                    (0x3050 + (1 * 6) + 3) /*0x3056*/: G1CR_ADDR,
                    (0x3050 + (2 * 6) + 2) /*0x3058*/: G2DR_ADDR,
                    (0x3050 + (2 * 6) + 3) /*0x3059*/: G2CR_ADDR,
                    (0x3050 + (3 * 6) + 2) /*0x305B*/: G3DR_ADDR,
                    (0x3050 + (3 * 6) + 3) /*0x305C*/: G3CR_ADDR,
                    (0x3050 + (4 * 6) + 2) /*0x305E*/: G4DR_ADDR,
                    (0x3050 + (4 * 6) + 3) /*0x305F*/: G4CR_ADDR,
                    (0x3050 + (5 * 6) + 2) /*0x3051*/: G5DR_ADDR,
                    (0x3050 + (5 * 6) + 3) /*0x3052*/: G5CR_ADDR,
                    (0x3050 + (6 * 6) + 2) /*0x3054*/: G6DR_ADDR,
                    (0x3050 + (6 * 6) + 3) /*0x3055*/: G6CR_ADDR,
                    (0x3050 + (7 * 6) + 2) /*0x3057*/: G7DR_ADDR,
                    (0x3050 + (7 * 6) + 3) /*0x3058*/: G7CR_ADDR,
                },
                insns: [
                    { AND R0, R0, #0 },
                    { ADD R0, R0, #(gpio_bools[0] as SignedWord) },
                    { STI R0, #0x4F }, // G0

                    { AND R0, R0, #0 },
                    { ADD R0, R0, #(state_to_word(Input))},
                    { STI R0, #0x4D },

                    { AND R1, R1, #0 },
                    { ADD R1, R1, #(gpio_bools[1] as SignedWord) },
                    { STI R1, #0x4F }, // G1

                    { AND R1, R1, #0 },
                    { ADD R1, R1, #(state_to_word(Input))},
                    { STI  R1, #0x4D },

                    { AND R2, R2, #0 },
                    { ADD R2, R2, #(gpio_bools[2] as SignedWord) },
                    { STI R2, #0x4F }, // G2

                    { AND R2, R2, #0 },
                    { ADD R2, R2, #(state_to_word(Input))},
                    { STI R2, #0x4D },

                    { AND R3, R3, #0 },
                    { ADD R3, R3, #(gpio_bools[3] as SignedWord) },
                    { STI R3, #0x4F }, // G3

                    { AND R3, R3, #0 },
                    { ADD R3, R3, #(state_to_word(Input))},
                    { STI R3, #0x4D },

                    { AND R4, R4, #0 },
                    { ADD R4, R4, #(gpio_bools[4] as SignedWord) },
                    { STI R4, #0x4F }, // G4

                    { AND R4, R4, #0 },
                    { ADD R4, R4, #(state_to_word(Input))},
                    { STI R4, #0x4D },

                    { AND R5, R5, #0 },
                    { ADD R5, R5, #(gpio_bools[5] as SignedWord) },
                    { STI R5, #0x4F }, // G5
                   
                    { AND R5, R5, #0 },
                    { ADD R5, R5, #(state_to_word(Input))},
                    { STI  R5, #0x4D },

                    { AND R6, R6, #0 },
                    { ADD R6, R6, #(gpio_bools[6] as SignedWord) },
                    { STI R6, #0x4F }, // G6
                    
                    { AND R6, R6, #0 },
                    { ADD R6, R6, #(state_to_word(Input))},
                    { STI  R6, #0x4D },

                    { AND R7, R7, #0 },
                    { ADD R7, R7, #(gpio_bools[7] as SignedWord) },
                    { STI R7, #0x4F }, // G7
                   
                    { AND R7, R7, #0 },
                    { ADD R7, R7, #(state_to_word(Input))},
                    { STI  R7, #0x4D },
                ],
                steps: GpioPin::NUM_PINS * 6,
                pre : |p| {
                    for pin in GPIO_PINS.iter() {
                        let _set = Gpio::set_state(p, *pin, Output);
                    }
                }
                post: |i| {
                    for (pin, pin_val) in GPIO_PINS.iter().zip(gpio_bools.iter()) {
                        let exp_pin_val = pin_val != &0;
                        let actual_pin_val = Gpio::read(i.get_peripherals(), *pin).unwrap();
                        eq!(actual_pin_val, exp_pin_val, "Gpio Pin {:?}\nExpected {}, got {}\nTest Case {:?}", *pin, exp_pin_val, actual_pin_val, gpio_vals);
                      
                    }


                }


            }
        }
    })}



    
}

mod interrupt {
    use super::*;
    use lc3_traits::peripherals::gpio::*;
   // Reading from pins in interrupt mode should already be covered; the only
   // thing left is to test that interrupts actually trigger.

   // Here are the variables:
   //   - rising edge or falling edge
   //   - in interrupt mode or some other mode (i.e. 3 other modes)

   // Interrupts should only trigger on rising edges AND when interrupts are
   // enabled AND when in interrupt mode. If we do an exhaustive test, this
   // is (2 * 4) ^ 8 = 16,777,216 states...
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

mod errors {
    use super::*;
    use lc3_traits::peripherals::gpio::{GpioWriteError, GpioReadError};
    use lc3_traits::error::Error;

    single_test! {
        gpio_write_error_disabled,
        prefill: { 0x3010: G0DR_ADDR },
        insns: [ { STI R0, #0xF } ],
        steps: 1,
        post: |i| { eq!(Error::from(GpioWriteError((G0, Disabled))), InstructionInterpreter::get_error(i).unwrap()); }
    }

    single_test! {
        gpio_write_error_input,
        prefill: { 0x3010: G0CR_ADDR, 0x3011: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b10 }, { STI R0, #0xD }, { STI R0, #0xD } ],
        steps: 4,
        regs: { R0: 0b10 },
        post: |i| { eq!(Error::from(GpioWriteError((G0, Input))), InstructionInterpreter::get_error(i).unwrap()); }
    }

    single_test! {
        gpio_read_error_disabled,
        prefill: { 0x3010: G0DR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        post: |i| { eq!(Error::from(GpioReadError((G0, Disabled))), InstructionInterpreter::get_error(i).unwrap()); }
    }

    single_test! {
        gpio_read_error_output,
        prefill: { 0x3010: G0CR_ADDR, 0x3011: G0DR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD }, { LDI R0, #0xD } ],
        steps: 4,
        regs: { R0: 0x8000 },
        post: |i| { eq!(Error::from(GpioReadError((G0, Output))), InstructionInterpreter::get_error(i).unwrap()); }
    }
}
