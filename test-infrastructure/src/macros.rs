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
        $(prefill_expr: { $(($addr_expr:expr): $val_expr:expr),* $(,)?},)?
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
            $(prefill_expr: { $(($addr_expr): $val_expr),* },)?
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
        $(prefill_expr: { $(($addr_expr:expr): $val_expr:expr),* $(,)?},)?
        insns: [ $({ $($insn:tt)* }),* $(,)?],
        $(steps: $steps:expr,)?
        regs: { $($r:tt: $v:expr),* $(,)?},
        memory: { $($addr:literal: $val:expr),* $(,)?} $(,)?
        $(post: |$peripherals_t:ident| $teardown:block)? $(,)?
        $(with os { $os:expr } @ $os_addr:expr)? $(,)?
    ) => {{
        use $crate::{Word, Reg, Instruction, ShareablePeripheralsShim, MemoryShim};
        use $crate::{PeripheralInterruptFlags, Interpreter, InstructionInterpreterPeripheralAccess};

        let flags = PeripheralInterruptFlags::new();

        fn setup_func_cast<'flags, S>(func: S, _f: &'flags PeripheralInterruptFlags) -> S
        where for<'p> S: FnOnce(&'p mut ShareablePeripheralsShim<'flags, '_>) {
            func
        }

        fn teardown_func_cast<'flags, T>(func: T, _f: &'flags PeripheralInterruptFlags) -> T
        where for<'i> T: FnOnce(&'i Interpreter<'flags, MemoryShim, ShareablePeripheralsShim<'flags, '_>>) {
            func
        }

        #[allow(unused)]
        let setup_func = setup_func_cast(|_p: &mut ShareablePeripheralsShim<'_, '_>| { }, &flags); // no-op if not specified
        $(let setup_func = setup_func_cast(|$peripherals_s: &mut ShareablePeripheralsShim<'_, '_>| $setup, &flags);)?

        #[allow(unused)]
        let teardown_func = teardown_func_cast(|_p: &Interpreter<'_, MemoryShim, ShareablePeripheralsShim<'_, '_>>| { }, &flags); // no-op if not specified
        $(let teardown_func = teardown_func_cast(|$peripherals_t: &Interpreter<'_, MemoryShim, ShareablePeripheralsShim<'_, '_>>| $teardown, &flags);)?

        // fn lifetime_hinter<'flags>(
        //     flags: &'flags PeripheralInterruptFlags,
        //     setup_func: impl FnOnce(&mut ShareablePeripheralsShim<'flags, 'static>),
        //     teardown_func: impl FnOnce(&Interpreter<'flags, MemoryShim, ShareablePeripheralsShim<'flags, 'static>>)
        // ) {
            #[allow(unused_mut)]
            let mut regs: [Option<Word>; Reg::NUM_REGS] = [None, None, None, None, None, None, None, None];
            $(regs[Into::<u8>::into($r) as usize] = Some($v);)*

            #[allow(unused_mut)]
            let mut checks: Vec<(Addr, Word)> = Vec::new();
            $(checks.push(($addr, $val));)*

            #[allow(unused_mut)]
            let mut prefill: Vec<(Addr, Word)> = Vec::new();
            $($(prefill.push(($addr_p, $val_p));)*)?
            $($(prefill.push(($addr_expr, $val_expr));)*)?

            #[allow(unused_mut)]
            let mut insns: Vec<Instruction> = Vec::new();
            $(insns.push(insn!($($insn)*));)*

            #[allow(unused)]
            let steps: Option<usize> = None;
            $(let steps: Option<usize> = Some($steps);)?

            #[allow(unused)]
            let os: Option<(MemoryShim, Addr)> = None;
            $(let os = Some(($os, $os_addr));)?

            interp_test_runner::<'_, MemoryShim, ShareablePeripheralsShim<'_, '_>, _, _>(
                prefill,
                insns,
                steps,
                regs,
                None,
                checks,
                // (|_|{}),
                // (|_|{}),
                // (|p: &mut ShareablePeripheralsShim<'flags, 'static>| (setup_func)(p)),
                // (|i: &Interpreter<'flags, MemoryShim, ShareablePeripheralsShim<'flags, 'static>>| (teardown_func)(i)),
                setup_func,
                teardown_func,
                &flags,
                os,
            );
        // }

        // lifetime_hinter(
        //     &flags,
        //     setup_func,
        //     teardown_func,
        // );
    }};
}
