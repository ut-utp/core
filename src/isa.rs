use super::Word;
use core::ops::Range;

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

trait Bits: Sized {
    fn bit(self, bit: u32) -> bool;

    fn b(self, bit: u32) -> bool {
        self.bit(bit)
    }

    fn bits(self, range: Range<u32>) -> usize;

    fn u8(self, range: Range<u32>) -> u8 {
        assert!(range.end - range.start <= 8);
        self.bits(range) as u8
    }

    fn u16(self, range: Range<u32>) -> u16 {
        assert!(range.end - range.start <= 16);
        self.bits(range) as u16
    }

    fn u32(self, range: Range<u32>) -> u32 {
        assert!(range.end - range.start <= 32);
        self.bits(range) as u32
    }
}

impl Bits for Word {
    fn bit(self, bit: u32) -> bool {
        ((self >> bit) & 1) == 1
    }

    fn bits(self, range: Range<u32>) -> usize {
        let mask = !(core::u16::MAX << (range.end - range.start));
        ((self >> range.start) & mask) as usize
    }
}

impl From<Word> for Instruction {
    // Assuming Word = u16; compile error if not.
    fn from(w: u16) -> Self {
        use Instruction::*;

        match w >> 12 {
            0b0000 => Br { n: w.b(11), z: w.b(10), p: w.b(9), offset9: w.u16(0..8) },
            0b0001 => match w.b(5) {
                false => AddReg { dr: w.u8(9..11), sr1: w.u8(6..8), sr2: w.u8(0..2) },
                true => AddImm { dr: w.u8(9..11), sr1: w.u8(6..8), imm5: w.u8(0..4) },
            },
            0b0010 => Ld { dr: w.u8(9..11), offset9: w.u16(0..8) },
            0b0011 => St { sr: w.u8(9..11), offset9: w.u16(0..8) },
            0b0100 => match w.bit(11) {
                true => Jsr { offset11: w.u16(0..10) },
                false => Jsrr { base: w.u8(6..8) },
            },
            0b0101 => match w.bit(5) {
                false => AndReg { dr: w.u8(9..11), sr1: w.u8(6..8), sr2: w.u8(0..2) },
                true => AndImm {  dr: w.u8(9..11), sr1: w.u8(6..8), imm5: w.u8(0..4) },
            },
            0b0110 => Ldr { dr: w.u8(9..11), base: w.u8(6..8), offset6: w.u8(0..5) },
            0b0111 => Str { dr: w.u8(9..11), base: w.u8(6..8), offset6: w.u8(0..5) },
            0b1000 => Rti,
            0b1001 => Not { dr: w.u8(9..11), sr: w.u8(6..8) },
            0b1010 => Ldi { dr: w.u8(9..11), offset9: w.u16(0..8) },
            0b1011 => Sti { sr: w.u8(9..11), offset9: w.u16(0..8) },
            0b1100 => match w.u8(6..8) {
                base @ 0..=6 => Jmp { base },
                7 => Ret,
                _ => unreachable!(),
            },
            0b1101 => unimplemented!(),
            0b1110 => Lea { dr: w.u8(9..11), offset9: w.u16(0..8) },
            0b1111 => Trap { trapvec: w.u8(0..7) },
            _ => unreachable!(),
        }
    }
}

impl From<Instruction> for Word {
    fn from(ins: Instruction) -> u16 {

    }
}
