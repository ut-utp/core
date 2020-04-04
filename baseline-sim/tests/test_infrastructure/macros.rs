#![macro_use]

// Setup func runs before anything is set; teardown func runs after everything
// is checked but the order shouldn't matter.
//
// `with os` takes a MemoryDump and a starting address to use as the entrypoint
#[macro_export]
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
        $(with os { $os:expr } @ $os_addr:expr)? $(,)?
    ) => {
    $(#[doc = $panics] #[should_panic])?
    #[test]
    fn $name() { with_larger_stack(/*Some(stringify!($name).to_string())*/ None, ||
        $crate::single_test_inner!(
            $(pre: |$peripherals_s| $setup,)?
            $(prefill: { $($addr_p: $val_p),* },)?
            insns: [ $({ $($insn)* }),* ],
            $(steps: $steps,)?
            regs: { $($r: $v),* },
            memory: { $($addr: $val),* }
            $(post: |$peripherals_t| $teardown)?
            $(with os { $os } @ $os_addr)?
        ));
    }};
}

#[macro_export]
macro_rules! single_test_inner {
    (   $(pre: |$peripherals_s:ident| $setup:block,)?
        $(prefill: { $($addr_p:literal: $val_p:expr),* $(,)?},)?
        insns: [ $({ $($insn:tt)* }),* $(,)?],
        $(steps: $steps:expr,)?
        regs: { $($r:tt: $v:expr),* $(,)?},
        memory: { $($addr:literal: $val:expr),* $(,)?} $(,)?
        $(post: |$peripherals_t:ident| $teardown:block)? $(,)?
        $(with os { $os:expr } @ $os_addr:expr)? $(,)?
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
        $(let steps: Option<usize> = Some($steps);)?

        let flags = PeripheralInterruptFlags::new();

        #[allow(unused)]
        let os = None;
        $(let os = Some(($os, $os_addr));)?

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
            &os,
        );
    }};
}
