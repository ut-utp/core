//! Home of the `lc3_sequence!` macro.

macro_rules! lc3_sequence {
    ($(|$panics:literal|)? $name:ident, insns: [ $({ $($insn:tt)* }),* ], lc3_insns: [ $($lc3_insn:expr),* ]) => {
    $(#[doc = $panics] #[should_panic])?
    #[test]
    fn $name() { $crate::with_larger_stack(None, || {
        use $crate::{Instruction, PeripheralInterruptFlags, MemoryShim, PeripheralsShim};

        #[allow(unused_mut)]
        let mut insns: Vec<Instruction> = Vec::new();
        $(insns.push(insn!($($insn)*));)*

        #[allow(unused_mut)]
        let mut lc3_insns: Vec<String> = Vec::new();
        $( lc3_insns.push($lc3_insn); )*

        let flags = PeripheralInterruptFlags::new();

        $crate::lc3tools::lc3tools_tester::<MemoryShim, PeripheralsShim, _, _>(
            Vec::new(),
            insns,
            lc3_insns,
            (|_p| {}), // (no-op)
            (|_p| {}), // (no-op)
            &flags,
            &None
        );
    })}};
}
