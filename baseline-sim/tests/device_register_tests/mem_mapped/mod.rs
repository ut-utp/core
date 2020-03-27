use lc3_isa::{insn, Addr, Instruction, Reg, Word, SignedWord};

use lc3_shims::memory::MemoryShim;
use lc3_shims::peripherals::PeripheralsShim;

use lc3_baseline_sim::interp::PeripheralInterruptFlags;

use pretty_assertions::assert_eq as eq;

#[path = "../../common.rs"]
mod common;
use common::interp_test_runner;

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

        let flags = PeripheralInterruptFlags::new();

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
            &flags,
        );
    }};
}

mod adc;
mod clock;
mod gpio;
mod pwm;
mod timers;

mod input;
mod output;
