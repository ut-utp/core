//! Home of the `lc3_sequence!` macro.

#[macro_export]
macro_rules! lc3_sequence {
    (
        $(|$panics:literal|)?
        $name:ident,
        insns: [ $({ $($insn:tt)* }),* $(,)?],
    ) => {
        $(#[doc = $panics] #[should_panic])?
        #[test]
        fn $name() -> std::io::Result<()> { $crate::with_larger_stack(None, || {
            use $crate::{
                Instruction, PeripheralInterruptFlags, MemoryShim, PeripheralsShim, insn
            };

            #[allow(unused_mut)]
            let mut insns: Vec<Instruction> = Vec::new();
            $(insns.push(insn!($($insn)*));)*

            let flags = PeripheralInterruptFlags::new();

            $crate::lc3tools::lc3tools_tester::<MemoryShim, PeripheralsShim, _, _>(
                insns,
                &flags,
                None
            )
        })}
    };
}
