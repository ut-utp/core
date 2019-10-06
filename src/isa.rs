use super::{Word};
use core::ops::RangeInclusive;

pub type RegNum = u8;

// Alternative way is to use repr(C) with bitfields.

pub enum Instruction {
    AddReg { dr: RegNum, sr1: RegNum, sr2: RegNum },
    AddImm { dr: RegNum, sr1: RegNum, imm5: u8 },
    AndReg { dr: RegNum, sr1: RegNum, sr2: RegNum },
    AndImm { dr: RegNum, sr1: RegNum, imm5: u8 },
    Br { n: bool, z: bool, p: bool, offset9: u16 },
    Jmp { base: RegNum },
    Jsr { offset11: u16 },
    Jsrr { base: RegNum },
    Ld { dr: RegNum, offset9: u16 },
    Ldi { dr: RegNum, offset9: u16 },
    Ldr { dr: RegNum, base: RegNum, offset6: u8 },
    Lea { dr: RegNum, offset9: u16 },
    Not { dr: RegNum, sr: RegNum },
    Ret,
    Rti,
    St { sr: RegNum, offset9: u16 },
    Sti { sr: RegNum, offset9: u16 },
    Str { dr: RegNum, base: RegNum, offset6: u8 },
    Trap { trapvec: u8 },
}

trait Bits {
    fn bit(self, bit: u32) -> bool;

    fn bits(self, range: RangeInclusive<u32>) -> Self;
}

impl Bits for u16 {
    fn bit(self, bit: u32) -> bool {
        ((self >> bit) & 1) == 1
    }

    fn bits(self, range: RangeInclusive<u32>) -> u16 {
        let mask = !(core::u16::MAX << (range.end() - range.start()));
        (self >> range.start()) & mask
    }
}

impl From<Word> for Instruction {

    // Assuming Word = u16; compile error if not.
    fn from(w: u16) -> Self {
        use Instruction::*;

        match w >> 12 {
            0b0000 => Br { n: w.bit(11), z: w.bit(10), p: w.bit(9), offset9: w.bits(0..=8) },
            0b0001 => match w.bit(5) {
                false => AddReg { dr: w.bits(9..=11) as u8, sr1: w.bits(6..=8) as u8, sr2: w.bits(0..=2) as u8 },
                true => AddImm { dr: w.bits(9..=11) as u8, sr1: w.bits(6..=8) as u8, imm5: w.bits(0..=4) as u8 },
            },
            0b0010 => Ld { dr: w.bits(9..=11) as u8, offset9: w.bits(0..=8) },
            0b0011 => St { sr: w.bits(9..=11) as u8, offset9: w.bits(0..=8) },
            0b0100 => match w.bit(11) {
                true => Jsr { offset11: w.bits(0..=10) },
                false => Jsrr { base: w.bits(6..=8) as u8 },
            },
            0b0101 => match w.bit(5) {
                false => AndReg { dr: w.bits(9..=11) as u8, sr1: w.bits(6..=8) as u8, sr2: w.bits(0..=2) as u8 },
                true => AndImm { dr: w.bits(9..=11) as u8, sr1: w.bits(6..=8) as u8, imm5: w.bits(0..=4) as u8 },
            },
            0b0110 => Ldr { dr: w.bits(9..=11) as u8, base: w.bits(6..=8) as u8, offset6: w.bits(0..=5) as u8 },
            0b0111 => Str { dr: w.bits(9..=11) as u8, base: w.bits(6..=8) as u8, offset6: w.bits(0..=5) as u8 },
            0b1000 => Rti,
            0b1001 => Not { dr: w.bits(9..=11) as u8, sr: w.bits(6..=8) as u8 },
            0b1010 => Ldi { dr: w.bits(9..=11) as u8, offset9: w.bits(0..=8) },
            0b1011 => Sti { sr: w.bits(9..=11) as u8, offset9: w.bits(0..=8) },
            0b1100 => match w.bits(6..=8) {
                reg @ 0..=6 => Jmp { base: reg as u8 },
                7 => Ret,
                _ => unreachable!(),
            },
            0b1101 => unimplemented!(),
            0b1110 => Lea { dr: w.bits(9..=11) as u8, offset9: w.bits(0..=8) },
            0b1111 => Trap { trapvec: w.bits(0..=7) as u8 },
            _ => unreachable!(),
        }
    }
}
