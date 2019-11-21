use lc3_isa::{Instruction, Reg};

#[macro_use] extern crate itertools;


fn all_add_reg() -> impl Iterator<Item = Instruction> {
    iproduct!(Reg::REGS.iter(), Reg::REGS.iter(), Reg::REGS.iter())
        .map(|(dr, sr1, sr2)| Instruction::new_add_reg(*dr, *sr1, *sr2))
}

fn all_add_imm() -> impl Iterator<Item = Instruction> {
    iproduct!(Reg::REGS.iter(), Reg::REGS.iter(), -16..=15)
        .map(|(dr, sr1, imm5)| Instruction::new_add_imm(*dr, *sr1, imm5))
}

fn all_and_reg() -> impl Iterator<Item = Instruction> {
    iproduct!(Reg::REGS.iter(), Reg::REGS.iter(), Reg::REGS.iter())
        .map(|(dr, sr1, sr2)| Instruction::new_and_reg(*dr, *sr1, *sr2))
}

fn all_and_imm() -> impl Iterator<Item = Instruction> {
    iproduct!(Reg::REGS.iter(), Reg::REGS.iter(), -16..=15)
        .map(|(dr, sr1, imm5)| Instruction::new_and_imm(*dr, *sr1, imm5))
}

fn all_br() -> impl Iterator<Item = Instruction> {}
fn all_jmp() -> impl Iterator<Item = Instruction> {}
fn all_jsr() -> impl Iterator<Item = Instruction> {}
fn all_jsrr() -> impl Iterator<Item = Instruction> {}
fn all_ld() -> impl Iterator<Item = Instruction>{}
fn all_ldi() -> impl Iterator<Item = Instruction> {}
fn all_ldr() -> impl Iterator<Item = Instruction> {}
fn all_lea() -> impl Iterator<Item = Instruction> {}
fn all_not() -> impl Iterator<Item = Instruction> {}
fn all_ret() -> impl Iterator<Item = Instruction> {}
fn all_rti() -> impl Iterator<Item = Instruction> {}
fn all_st() -> impl Iterator<Item = Instruction> {}
fn all_sti() -> impl Iterator<Item = Instruction> {}
fn all_str() -> impl Iterator<Item = Instruction> {}
fn all_trap() -> impl Iterator<Item = Instruction> {}

fn all_insns() -> impl Iterator<Item = Instruction> {
    let insns: Vec<Instruction> = Vec::new();

    let iter = std::iter::empty
        .chain(all_add_imm())
        .chain(all_and_reg())
        .chain(all_and_imm())
        .chain(all_br())
        .chain(all_jmp())
        .chain(all_jsr())
        .chain(all_jsrr())
        .chain(all_ld())
        .chain(all_ldi())
        .chain(all_ldr())
        .chain(all_lea())
        .chain(all_not())
        .chain(all_ret())
        .chain(all_rti())
        .chain(all_st())
        .chain(all_sti())
        .chain(all_str())
        .chain(all_trap());



    // for i in 0..19 {
    //     match i {
    //         0 => all_add_reg(&mut insns),
    //         1 => all_add_imm(&mut insns),
    //         2 => all_and_reg(&mut insns),
    //         3 => all_and_imm(&mut insns),
    //         4 => all_br(&mut insns),
    //         5 => all_jmp(&mut insns),
    //         6 => all_jsr(&mut insns),
    //         7 => all_jsrr(&mut insns),
    //         8 => all_ld(&mut insns),
    //         9 => all_ldi(&mut insns),
    //         10 => all_ldr(&mut insns),
    //         11 => all_lea(&mut insns),
    //         12 => all_not(&mut insns),
    //         13 => all_ret(&mut insns),
    //         14 => all_rti(&mut insns),
    //         15 => all_st(&mut insns),
    //         16 => all_sti(&mut insns),
    //         17 => all_str(&mut insns),
    //         18 => all_trap(&mut insns),
    //     }
    // }

    insns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
    }
}
