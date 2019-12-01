use lc3_isa::{Instruction, Reg};

use pretty_assertions::assert_eq;

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

#[test]
fn br_display() {
    use Instruction::*;

    fn check(n: bool, z: bool, p: bool, offset9: i16, text: &str) {
        assert_eq!(format!("{}", Br { n, z, p, offset9 }), text);
    }

    const T: bool = true;
    const F: bool = false;

    check(F, F, F, -5, "BR    #-5"); // TODO: unrepresentable?
    check(T, F, F, -5, "BRn   #-5");
    check(F, F, T, -5, "BRp   #-5");
    check(T, F, T, -5, "BRnp  #-5");
    check(T, T, T, -5, "BRnzp #-5");
}
