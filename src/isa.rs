use super::Word;
use core::ops::Range;

pub type RegNum = u8;

// Alternative way is to use repr(C) with bitfields.

#[rustfmt::skip]
pub enum Instruction {
    AddReg { dr: RegNum, sr1: RegNum, sr2: RegNum }, // RRR
    AddImm { dr: RegNum, sr1: RegNum, imm5: i8 },    // RR5
    AndReg { dr: RegNum, sr1: RegNum, sr2: RegNum }, // RRR
    AndImm { dr: RegNum, sr1: RegNum, imm5: i8 },    // RR5
    Br { n: bool, z: bool, p: bool, offset9: i16 },  // nzp9
    Jmp { base: RegNum },                            // B
    Jsr { offset11: i16 },                           // a
    Jsrr { base: RegNum },                           // B
    Ld { dr: RegNum, offset9: i16 },                 // R9
    Ldi { dr: RegNum, offset9: i16 },                // R9
    Ldr { dr: RegNum, base: RegNum, offset6: i8 },   // RR6
    Lea { dr: RegNum, offset9: i16 },                // R9
    Not { dr: RegNum, sr: RegNum },                  // RR
    Ret,                                             //
    Rti,                                             //
    St { sr: RegNum, offset9: i16 },                 // R9
    Sti { sr: RegNum, offset9: i16 },                // R9
    Str { sr: RegNum, base: RegNum, offset6: i8 },   // RR6
    Trap { trapvec: u8 },                            // 8
}

trait Bits: Sized + Copy {
    fn bit(self, bit: u32) -> bool;

    fn b(self, bit: u32) -> bool {
        self.bit(bit)
    }

    fn bits(self, range: Range<u32>) -> usize;

    fn u8(self, range: Range<u32>) -> u8 {
        assert!(range.end - range.start <= 8);
        self.bits(range) as u8
    }

    fn i8(self, range: Range<u32>) -> i8 {
        assert!(range.end - range.start <= 8);

        (if self.bit(range.end) {
            core::u8::MAX << (range.end - range.start)
        } else {
            0
        } | self.u8(range)) as i8
    }

    fn u16(self, range: Range<u32>) -> u16 {
        assert!(range.end - range.start <= 16);
        self.bits(range) as u16
    }

    fn i16(self, range: Range<u32>) -> i16 {
        assert!(range.end - range.start <= 16);

        (if self.bit(range.end) {
            core::u16::MAX << (range.end - range.start)
        } else {
            0
        } | self.u16(range)) as i16
    }

    fn u32(self, range: Range<u32>) -> u32 {
        assert!(range.end - range.start <= 32);
        self.bits(range) as u32
    }

    fn i32(self, range: Range<u32>) -> i32 {
        assert!(range.end - range.start <= 32);

        (if self.bit(range.end) {
            core::u32::MAX << (range.end - range.start)
        } else {
            0
        } | self.u32(range)) as i32
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

        #[rustfmt::skip]
        match w >> 12 {
            0b0000 => Br { n: w.b(11), z: w.b(10), p: w.b(9), offset9: w.i16(0..8) },
            0b0001 => match w.b(5) {
                false => AddReg { dr: w.u8(9..11), sr1: w.u8(6..8), sr2: w.u8(0..2) },
                true => AddImm { dr: w.u8(9..11), sr1: w.u8(6..8), imm5: w.i8(0..4) },
            },
            0b0010 => Ld { dr: w.u8(9..11), offset9: w.i16(0..8) },
            0b0011 => St { sr: w.u8(9..11), offset9: w.i16(0..8) },
            0b0100 => match w.bit(11) {
                true => Jsr { offset11: w.i16(0..10) },
                false => Jsrr { base: w.u8(6..8) },
            },
            0b0101 => match w.bit(5) {
                false => AndReg { dr: w.u8(9..11), sr1: w.u8(6..8), sr2: w.u8(0..2) },
                true => AndImm {  dr: w.u8(9..11), sr1: w.u8(6..8), imm5: w.i8(0..4) },
            },
            0b0110 => Ldr { dr: w.u8(9..11), base: w.u8(6..8), offset6: w.i8(0..5) },
            0b0111 => Str { sr: w.u8(9..11), base: w.u8(6..8), offset6: w.i8(0..5) },
            0b1000 => Rti,
            0b1001 => Not { dr: w.u8(9..11), sr: w.u8(6..8) },
            0b1010 => Ldi { dr: w.u8(9..11), offset9: w.i16(0..8) },
            0b1011 => Sti { sr: w.u8(9..11), offset9: w.i16(0..8) },
            0b1100 => match w.u8(6..8) {
                base @ 0..=6 => Jmp { base },
                7 => Ret,
                _ => unreachable!(),
            },
            0b1101 => unimplemented!(),
            0b1110 => Lea { dr: w.u8(9..11), offset9: w.i16(0..8) },
            0b1111 => Trap { trapvec: w.u8(0..7) },
            16..=core::u16::MAX => unreachable!(),
        }
    }
}

impl From<Instruction> for Word {
    #[rustfmt::skip]
    fn from(ins: Instruction) -> u16 {
        #![allow(non_snake_case)]
        use Instruction::*;

        fn Op(op: u8) -> Word { ((op as u16) & 0b1111) << 12 }
        fn Dr(dr: RegNum) -> Word { ((dr as u16) & 0b111) << 8 }
        fn Sr1(sr1: RegNum) -> Word { ((sr1 as u16) & 0b111) << 5 }
        fn Sr2(sr2: RegNum) -> Word { (sr2 as u16) & 0b111 }
        fn Imm5(imm5: i8) -> Word { (1 << 5) | ((imm5 as u16) & 0b11111) }
        fn N(n: bool) -> Word { (n as u16) << 10 }
        fn Z(z: bool) -> Word { (z as u16) << 9 }
        fn P(p: bool) -> Word { (p as u16) << 8 }
        fn O9(offset9: i16) -> Word { (offset9 as u16) & 0b111111111 }
        fn O11(offset11: i16) -> Word { (1 << 11) | ((offset11 as u16) & 0x7FF) }
        fn Base(base: RegNum) -> Word { Sr1(base) }
        fn O6(offset6: i8) -> Word { (offset6 as u16) & 0b111111 }
        fn Sr(sr: RegNum) -> Word { Sr1(sr) | 0b111111 }
        fn Trapvec(trapvec: u8) -> Word { (trapvec as u16) & 0xFF }

        match ins {
            AddReg { dr, sr1, sr2 }   => Op(0b0001) | Dr(dr) | Sr1(sr1)   | Sr2(sr2)   ,
            AddImm { dr, sr1, imm5 }  => Op(0b0001) | Dr(dr) | Sr1(sr1)   | Imm5(imm5) ,
            AndReg { dr, sr1, sr2 }   => Op(0b0101) | Dr(dr) | Sr1(sr1)   | Sr2(sr2)   ,
            AndImm { dr, sr1, imm5 }  => Op(0b0001) | Dr(dr) | Sr1(sr1)   | Imm5(imm5) ,
            Br { n, z, p, offset9 }   => Op(0b0000) | N(n) | Z(z) | P(p)  | O9(offset9),
            Jmp { base }              => Op(0b1100)          | Base(base)              ,
            Jsr { offset11: offset }  => Op(0b0100)                       | O11(offset),
            Jsrr { base }             => Op(0b0100)          | Base(base)              ,
            Ld { dr, offset9 }        => Op(0b0010) | Dr(dr)              | O9(offset9),
            Ldi { dr, offset9 }       => Op(0b1010) | Dr(dr)              | O9(offset9),
            Ldr { dr, base, offset6 } => Op(0b0110) | Dr(dr) | Base(base) | O6(offset6),
            Lea { dr, offset9 }       => Op(0b1110) | Dr(dr)              | O9(offset9),
            Not { dr, sr }            => Op(0b1001) | Dr(dr) | Sr(sr)                  ,
            Ret                       => Op(0b1100)          | Base(7)                 ,
            Rti                       => Op(0b1000)                                    ,
            St { sr, offset9 }        => Op(0b0011) | Dr(sr)              | O9(offset9),
            Sti { sr, offset9 }       => Op(0b1011) | Dr(sr)              | O9(offset9),
            Str { sr, base, offset6 } => Op(0b0111) | Dr(sr) | Base(base) | O6(offset6),
            Trap { trapvec }          => Op(0b0111)          | Trapvec(trapvec)        ,
        }
    }
}
