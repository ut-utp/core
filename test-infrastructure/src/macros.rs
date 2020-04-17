#![macro_use]

#[doc(inline)]
pub use crate::single_test;

#[doc(inline)]
pub use crate::single_test_inner;

// Setup func runs before anything is set; teardown func runs after everything
// is checked but the order shouldn't matter.
//
// `with os` takes a MemoryDump and a starting address to use as the entrypoint
#[macro_export]
macro_rules! single_test {
    ($(|$panics:literal|)?
        $name:ident,
        $(prefill: { $($addr_p:literal: $val_p:expr),* $(,)?} $(,)?)?
        $(prefill_expr: { $(($addr_expr:expr): $val_expr:expr),* $(,)?} $(,)?)?
        insns: [ $({ $($insn:tt)* }),* $(,)?] $(,)?
        $(steps: $steps:expr,)?
        $(regs: { $($r:tt: $v:expr),* $(,)?} $(,)?)?
        $(memory: { $($addr:literal: $val:expr),* $(,)?} $(,)?)?
        $(with io peripherals: { source as $inp:ident, sink as $out:ident } $(,)?)?
        $(with custom peripherals: $custom_per:block -> [$custom_per_ty:tt] $(,)?)?
        $(pre: |$peripherals_s:ident| $setup:block $(,)?)?
        $(post: |$peripherals_t:ident| $teardown:block $(,)?)?
        $(with os { $os:expr } @ $os_addr:expr $(,)?)?
    ) => {
    $(#[doc = $panics] #[should_panic])?
    #[test]
    fn $name() { $crate::with_larger_stack(/*Some(stringify!($name).to_string())*/ None, ||
        $crate::single_test_inner!(
            $(prefill: { $($addr_p: $val_p),* },)?
            $(prefill_expr: { $(($addr_expr): $val_expr),* },)?
            insns: [ $({ $($insn)* }),* ],
            $(steps: $steps,)?
            $(regs: { $($r: $v),* },)?
            $(memory: { $($addr: $val),* },)?
            $(with io peripherals: { source as $inp, sink as $out },)?
            $(with custom peripherals: $custom_per -> [$custom_per_ty],)?
            $(pre: |$peripherals_s| $setup,)?
            $(post: |$peripherals_t| $teardown,)?
            $(with os { $os } @ $os_addr,)?
        ));
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __perip_type {
    // ($regular:ty | io: $($_io:literal $io_ty:ty)? | custom: $($custom_ty:ty)?) => { };
    ($regular:ty | io:                      | custom:              ) => { $regular };
    ($regular:ty | io:                      | custom: $custom_ty:ty) => { $custom_ty };
    ($regular:ty | io: $_io:ident $io_ty:ty | custom:              ) => { $io_ty };
    ($regular:ty | io: $_io:ident $io_ty:ty | custom: $custom_ty:ty) => { $custom_ty };
}

#[macro_export]
macro_rules! single_test_inner {
    (   $(prefill: { $($addr_p:literal: $val_p:expr),* $(,)?} $(,)?)?
        $(prefill_expr: { $(($addr_expr:expr): $val_expr:expr),* $(,)?} $(,)?)?
        insns: [ $({ $($insn:tt)* }),* $(,)?]  $(,)?
        $(steps: $steps:expr,)?
        $(regs: { $($r:tt: $v:expr),* $(,)?} $(,)?)?
        $(memory: { $($addr:literal: $val:expr),* $(,)?} $(,)?)?
        $(with io peripherals: { source as $inp:ident, sink as $out:ident } $(,)?)?
        $(with custom peripherals: $custom_per:block -> [$custom_per_ty:tt] $(,)?)?
        $(pre: |$peripherals_s:ident| $setup:block $(,)?)?
        $(post: |$peripherals_t:ident| $teardown:block $(,)?)?
        $(with os { $os:expr } @ $os_addr:expr $(,)?)?
    ) => {{
        #[allow(unused_imports)]
        use super::*;

        #[allow(unused_imports)]
        use $crate::{
            Addr, Word, Reg, Instruction, insn, Reg::*,
            ShareablePeripheralsShim, MemoryShim, SourceShim, new_shim_peripherals_set,
            PeripheralInterruptFlags, Interpreter, InstructionInterpreterPeripheralAccess,
        };

        #[allow(unused_imports)]
        use std::sync::{Arc, Mutex};

        let flags = PeripheralInterruptFlags::new();

        type Per<'int, 'io> = $crate::__perip_type! {
            ShareablePeripheralsShim<'int, 'io>
            | io: $($inp ShareablePeripheralsShim<'int, 'io>)?
            | custom: $($custom_per_ty<'int, 'io>)?
        };

        #[allow(unused_mut)]
        let mut regs: [Option<Word>; Reg::NUM_REGS] = [None, None, None, None, None, None, None, None];
        $($(regs[Into::<u8>::into($r) as usize] = Some($v);)*)?

        #[allow(unused_mut)]
        let mut checks: Vec<(Addr, Word)> = Vec::new();
        $($(checks.push(($addr, $val));)*)?

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

        #[allow(unused)]
        let custom_peripherals: Option<Per<'_, '_>> = None;

        $(
            #[allow(unused)]
            let $inp = Arc::new(SourceShim::new());
            #[allow(unused)]
            let $out = Arc::new(Mutex::new(Vec::<u8>::new()));

            let (custom_peripherals, _, _): (Per<'_, '_>, _, _) =
                new_shim_peripherals_set(&$inp, &$out);
            #[allow(unused)]
            let custom_peripherals = Some(custom_peripherals);
        )?

        $(
            let custom_peripherals = $custom_per;
            let custom_peripherals = Some(custom_peripherals);
        )?

        fn setup_func_cast<'flags, S>(func: S, _f: &'flags PeripheralInterruptFlags) -> S
        where for<'p> S: FnOnce(&'p mut Per<'flags, '_>) {
            func
        }

        fn teardown_func_cast<'flags, T>(func: T, _f: &'flags PeripheralInterruptFlags) -> T
        where for<'i> T: FnOnce(&'i Interpreter<'flags, MemoryShim, Per<'flags, '_>>) {
            func
        }

        #[allow(unused)]
        let setup_func = setup_func_cast(|_p: &mut Per<'_, '_>| { }, &flags); // no-op if not specified
        $(let setup_func = setup_func_cast(|$peripherals_s: &mut Per<'_, '_>| $setup, &flags);)?

        #[allow(unused)]
        let teardown_func = teardown_func_cast(|_p: &Interpreter<'_, MemoryShim, Per<'_, '_>>| { }, &flags); // no-op if not specified
        $(let teardown_func = teardown_func_cast(|$peripherals_t: &Interpreter<'_, MemoryShim, Per<'_, '_>>| $teardown, &flags);)?


        $crate::interp_test_runner::<'_, MemoryShim, Per<'_, '_>, _, _>(
            prefill,
            insns,
            steps,
            regs,
            None,
            checks,
            setup_func,
            teardown_func,
            &flags,
            os,
            custom_peripherals,
        );
    }};
}

#[cfg(test)]
mod smoke_tests {
    use super::*;
    use crate::InstructionInterpreter;

    use lc3_traits::peripherals::stubs::PeripheralsStub;
    use lc3_traits::peripherals::clock::Clock;

    use std::default::Default;
    use std::io::Read;

    // // Just some compile tests:
    fn io_perip() {
        single_test_inner! {
            insns: [ { LDI R0, #0xF }, ],
            with io peripherals: { source as inp, sink as out },
        }
    }

    fn io_perip_used() {
        single_test_inner! {
            insns: [{ LDI R0, #0xF }],
            with io peripherals: { source as inp, sink as out },
            pre: |_p| {
                inp.push('a');
            }
            post: |_i| {
                inp.push('b');
            }
        }
    }

    #[allow(unused_lifetimes)]
    type PeripheralsStubAlias<'int, 'io> = PeripheralsStub<'int>;

    fn custom_perip() {
        single_test_inner! {
            insns: [],
            with custom peripherals: { PeripheralsStub::default() } -> [PeripheralsStubAlias],
        }
    }

    // These should not compile since `with io peripherals` and `with custom peripherals`
    // want different types for the peripherals.
    /*
    fn io_and_custom_perip() {
        single_test_inner! {
            insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
            with io peripherals: { source as inp, sink as out },
            with custom peripherals: { PeripheralsStub::default() } -> [PeripheralsStubAlias],
            pre: |p| {
                p.get_milliseconds();
            }
            post: |i| {
                i.get_pc();
            }
        }
    }

    fn io_and_custom_perip_used() {
        single_test_inner! {
            insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
            with io peripherals: { source as inp, sink as out },
            with custom peripherals: { PeripheralsStub::default() } -> [PeripheralsStubAlias],
            pre: |p| {
                inp.push('a');
                out.lock().unwrap().read_to_string();
                p.get_milliseconds();
            }
            post: |i| {
                i.get_pc();
                inp.push('b');
                out.lock().unwrap().read_to_string();
            }
        }
    }
    */

    fn all_with_custom_and_commas() {
        single_test_inner! {
            prefill: {
                0x3000: 2_109 * 1,
            },
            prefill_expr: {
                (0x3000 + 1): 'f' as Word,
            },
            insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ],
            steps: 890 * 0x789,
            regs: {
                R0: 0 * 7 + 3,
                R1: 1,
                R2: 2,
                R3: 3,
            },
            memory: {
                0x3000: 2343 - 234,
            },
            with custom peripherals: { PeripheralsStub::default() } -> [PeripheralsStubAlias],
            pre: |p| {
                p.get_milliseconds();
            },
            post: |i| {
                i.get_pc();
            },
        }
    }

    fn all_with_io_and_no_commas() {
        single_test_inner! {
            prefill: { 0x3000: 2_109 * 1, }
            prefill_expr: { (0x3000 + 1): 'f' as Word, }
            insns: [ { AND R0, R0, #0 }, { ADD R0, R0, #0b01 }, { STI R0, #0xD } ]
            steps: 890 * 0x789,
            regs: { R0: 0 * 7 + 3, R1: 1 }
            memory: { 0x3000: 2343 - 234 }
            with io peripherals: { source as inp, sink as out },
            pre: |p| {
                inp.push('a');

                out.lock().unwrap().drain(..);

                p.get_milliseconds();
            }
            post: |i| {
                i.get_pc();
                inp.push('b');

                let mut s = String::new();

                <&[u8]>::read_to_string(&mut out.lock().unwrap().as_ref(), &mut s);
            }
        }
    }

    fn thread_safe() {
        use lc3_application_support::io_peripherals::{InputSink, OutputSource};

        single_test_inner! {
            insns: [],
            with io peripherals: { source as inp, sink as out },
            pre: |p| {
                let inp = inp.clone();
                let out = out.clone();

                std::thread::spawn(move || {
                    loop {
                        inp.put_char('a');
                    }
                });

                std::thread::spawn(move || {
                    loop { out.get_chars().map(|o| println!("{}", o)); }
                });
            }
        }
    }
}
