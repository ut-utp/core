use lc3_baseline_sim::interp::{Interpreter, InstructionInterpreter, InstructionInterpreterPeripheralAccess};

mod reset {
    use super::*;
    use lc3_isa::Reg::*;
    use lc3_test_infrastructure::{MemoryShim, PeripheralsShim};

    #[test]
    fn reset_test() {
        let mut interp: Interpreter<MemoryShim, PeripheralsShim> = Interpreter::default();
        InstructionInterpreter::reset(&mut interp);
        assert_eq!(InstructionInterpreter::get_register(&interp, R0), 0);


    }
}