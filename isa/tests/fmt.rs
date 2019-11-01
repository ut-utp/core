use lc3_isa::{Instruction, Reg};

#[test]
fn registers() {
    use Reg::*;

    fn check(reg: Reg, text: &str) {
        assert_eq!(format!("{}", reg), text);
    }

    check(R0, "R0");
    check(R1, "R1");
    check(R2, "R2");
    check(R3, "R3");
    check(R4, "R4");
    check(R5, "R5");
    check(R6, "R6");
    check(R7, "R7");
}

