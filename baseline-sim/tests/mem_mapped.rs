use lc3_baseline_sim::*;
use lc3_isa::{insn, Addr, Instruction, Reg, Word};
use lc3_traits::memory::Memory;
use lc3_traits::peripherals::Peripherals;

use lc3_baseline_sim::interp::{InstructionInterpreter, Interpreter, MachineState};

use lc3_shims::memory::MemoryShim;
use lc3_shims::peripherals::PeripheralsShim;

#[path = "common.rs"]
mod common;

#[cfg(test)]
mod gpio_mem_mapped {
    use super::*;
    use pretty_assertions::assert_eq;

    use common::interp_test_runner;

    use Reg::*;

    // Setup func runs before anything is set; teardown func runs after everything is
    // checked but the order shouldn't matter.
    macro_rules! single_test {
        ($(|$panics:literal|)?
            $name:ident,
            $(pre: |$peripherals_s:ident| $setup:block,)?
            $(prefill: { $($addr_p:literal: $val_p:expr),* },)?
            insns: [ $({ $($insn:tt)* }),* ],
            $(steps: $steps:expr,)?
            regs: { $($r:tt: $v:expr),* },
            memory: { $($addr:literal: $val:expr),* } $(,)?
            $(post: |$peripherals_t:ident| $teardown:block)? $(,)?
        ) => {
        $(#[doc = $panics] #[should_panic])?
        #[test]
        fn $name() {

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

    use lc3_traits::peripherals::gpio::{Gpio, GpioPin, GpioState};
    use lc3_baseline_sim::mem_mapped::{G0CR_ADDR, G1CR_ADDR, G2CR_ADDR, G3CR_ADDR, G4CR_ADDR, G5CR_ADDR, G6CR_ADDR, G7CR_ADDR};

    single_test! {
        gpio_cr_pin0_read_output,
        pre: |p| { Gpio::set_state(p, GpioPin::G0, GpioState::Output).unwrap(); },
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { LDI R0, #0xF } ],
        steps: 1,
        regs: { R0: 0b01 },
        memory: { }
    }

    single_test! {
        gpio_cr_pin0_set_output_invalid,
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b1101 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b1101 },
        memory: { },
        post: |p| { assert_eq!(GpioState::Output, Gpio::get_state(p, GpioPin::G0)); }
    }

    single_test! {
        gpio_cr_pin0_set_output_valid,
        prefill: { 0x3010: G0CR_ADDR },
        insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
        steps: 3,
        regs: { R0: 0b01 },
        memory: { },
        post: |p| { assert_eq!(GpioState::Output, Gpio::get_state(p, GpioPin::G0)); }
    }
}
