use super::Word;

use core::cmp::Ordering;
use core::convert::{TryFrom, TryInto};
use core::ops::Range;

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PriorityLevel { PL0, PL1, PL2, PL3, PL4, PL5, PL6, PL7 }

// TODO: ditch the next four things once the macro is written:
impl PriorityLevel {
    pub const NUM_LEVELS: usize = 8;

    pub const LEVELS: [PriorityLevel; PriorityLevel::NUM_LEVELS] = {
        use PriorityLevel::*;
        [PL0, PL1, PL2, PL3, PL4, PL5, PL6, PL7]
    };
}

impl TryFrom<u8> for PriorityLevel {
    type Error = ();

    fn try_from(num: u8) -> Result<Self, ()> {
        use PriorityLevel::*;

        if Into::<usize>::into(num) < Self::NUM_LEVELS {
            Ok(match num {
                0 => PL0,
                1 => PL1,
                2 => PL2,
                3 => PL3,
                4 => PL4,
                5 => PL5,
                6 => PL6,
                7 => PL7,
                _ => unreachable!(),
            })
        } else {
            Err(())
        }
    }
}

impl From<PriorityLevel> for u8 {
    fn from(pl: PriorityLevel) -> u8 {
        use PriorityLevel::*;

        match pl {
            PL0 => 0,
            PL1 => 1,
            PL2 => 2,
            PL3 => 3,
            PL4 => 4,
            PL5 => 5,
            PL6 => 6,
            PL7 => 7,
        }
    }
}

impl From<&PriorityLevel> for u8 {
    fn from(pl: &PriorityLevel) -> u8 {
        <PriorityLevel as Into<u8>>::into(*pl)
    }
}

impl PartialOrd<PriorityLevel> for PriorityLevel {
    fn partial_cmp(&self, other: &PriorityLevel) -> Option<Ordering> {
        // Higher priorities have 'greater' precedence and are 'more important'.
        Into::<u8>::into(self).partial_cmp(&Into::<u8>::into(other))
    }
}

impl Ord for PriorityLevel {
    fn cmp(&self, other: &PriorityLevel) -> Ordering {
        // Higher priorities have 'greater' precedence and are 'more important'.
        Into::<u8>::into(self).cmp(&Into::<u8>::into(other))
    }
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reg { R0, R1, R2, R3, R4, R5, R6, R7 }

// TODO: ditch these next four things once we write the macro...
impl Reg {
    pub const NUM_REGS: usize = 8;

    pub const REGS: [Reg; Reg::NUM_REGS] = {
        use Reg::*;
        [R0, R1, R2, R3, R4, R5, R6, R7]
    };
}

impl TryFrom<u8> for Reg {
    type Error = ();

    fn try_from(num: u8) -> Result<Self, ()> {
        use Reg::*;

        if Into::<usize>::into(num) < Self::NUM_REGS {
            Ok(match num {
                0 => R0,
                1 => R1,
                2 => R2,
                3 => R3,
                4 => R4,
                5 => R5,
                6 => R6,
                7 => R7,
                _ => unreachable!(),
            })
        } else {
            Err(())
        }
    }
}

impl From<Reg> for u8 {
    fn from(reg: Reg) -> u8 {
        use Reg::*;

        match reg {
            R0 => 0,
            R1 => 1,
            R2 => 2,
            R3 => 3,
            R4 => 4,
            R5 => 5,
            R6 => 6,
            R7 => 7,
        }
    }
}

impl From<&Reg> for u8 {
    fn from(reg: &Reg) -> u8 {
        <Reg as Into<u8>>::into(*reg)
    }
}

// Alternative way is to use repr(C) with bitfields.

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    AddReg { dr: Reg, sr1: Reg, sr2: Reg },         // RRR
    AddImm { dr: Reg, sr1: Reg, imm5: i16 },        // RR5
    AndReg { dr: Reg, sr1: Reg, sr2: Reg },         // RRR
    AndImm { dr: Reg, sr1: Reg, imm5: i16 },        // RR5
    Br { n: bool, z: bool, p: bool, offset9: i16 }, // nzp9
    Jmp { base: Reg },                              // B
    Jsr { offset11: i16 },                          // a
    Jsrr { base: Reg },                             // B
    Ld { dr: Reg, offset9: i16 },                   // R9
    Ldi { dr: Reg, offset9: i16 },                  // R9
    Ldr { dr: Reg, base: Reg, offset6: i16 },        // RR6
    Lea { dr: Reg, offset9: i16 },                  // R9
    Not { dr: Reg, sr: Reg },                       // RR
    Ret,                                            //
    Rti,                                            //
    St { sr: Reg, offset9: i16 },                   // R9
    Sti { sr: Reg, offset9: i16 },                  // R9
    Str { sr: Reg, base: Reg, offset6: i16 },        // RR6
    Trap { trapvec: u8 },                           // 8
}

impl Instruction {
    pub fn sets_condition_codes(&self) -> bool {
        use Instruction::*;

        match self {
            AddReg { .. }
            | AddImm { .. }
            | AndReg { .. }
            | AndImm { .. }
            | Ld { .. }
            | Ldi { .. }
            | Ldr { .. }
            | Not { .. } => true,
            Br { .. }
            | Jmp { .. }
            | Jsr { .. }
            | Jsrr { .. }
            | Lea { .. }
            | Ret
            | Rti
            | St { .. }
            | Sti { .. }
            | Str { .. }
            | Trap { .. } => false,
        }
    }
}

pub trait Bits: Sized + Copy {
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

    fn word(self, range: Range<u32>) -> Word {
        self.u16(range)
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

    fn reg(self, lowest_bit: u32) -> Reg {
        self.u8(lowest_bit..(lowest_bit + 2)).try_into().unwrap()
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

impl TryFrom<Word> for Instruction {
    type Error = Word;

    // Assuming Word = u16; compile error if not.
    #[rustfmt::skip]
    fn try_from(w: u16) -> Result<Self, u16> {
        use Instruction::*;
        use Reg::*;

        let op_code: u8 = (w >> 12).try_into().unwrap();

        if op_code == 0b1101 {
            return Err(w)
        }

        Ok(match op_code {
            0b0000 => Br { n: w.b(11), z: w.b(10), p: w.b(9), offset9: w.i16(0..8) },
            0b0001 => match w.b(5) {
                false => AddReg { dr: w.reg(9), sr1: w.reg(6), sr2: w.reg(0) },
                true => AddImm { dr: w.reg(9), sr1: w.reg(6), imm5: w.i16(0..4) },
            },
            0b0010 => Ld { dr: w.reg(9), offset9: w.i16(0..8) },
            0b0011 => St { sr: w.reg(9), offset9: w.i16(0..8) },
            0b0100 => match w.bit(11) {
                true => Jsr { offset11: w.i16(0..10) },
                false => Jsrr { base: w.reg(6) },
            },
            0b0101 => match w.bit(5) {
                false => AndReg { dr: w.reg(9), sr1: w.reg(6), sr2: w.reg(0) },
                true => AndImm {  dr: w.reg(9), sr1: w.reg(6), imm5: w.i16(0..4) },
            },
            0b0110 => Ldr { dr: w.reg(9), base: w.reg(6), offset6: w.i16(0..5) },
            0b0111 => Str { sr: w.reg(9), base: w.reg(6), offset6: w.i16(0..5) },
            0b1000 => Rti,
            0b1001 => Not { dr: w.reg(9), sr: w.reg(6) },
            0b1010 => Ldi { dr: w.reg(9), offset9: w.i16(0..8) },
            0b1011 => Sti { sr: w.reg(9), offset9: w.i16(0..8) },
            0b1100 => match w.reg(6) {
                R7 => Ret,
                base => Jmp { base },
            },
            0b1110 => Lea { dr: w.reg(9), offset9: w.i16(0..8) },
            0b1111 => Trap { trapvec: w.u8(0..7) },
            0b1101 | 16..=core::u8::MAX => unreachable!(),
        })
    }
}

impl From<Instruction> for Word {
    #[rustfmt::skip]
    fn from(ins: Instruction) -> u16 {
        #![allow(non_snake_case)]
        use Instruction::*;

        fn Op(op: u8) -> Word { ((op as u16) & 0b1111) << 12 }
        fn Dr(dr: Reg) -> Word { ((dr as u16) & 0b111) << 8 }
        fn Sr1(sr1: Reg) -> Word { ((sr1 as u16) & 0b111) << 5 }
        fn Sr2(sr2: Reg) -> Word { (sr2 as u16) & 0b111 }
        fn Imm5(imm5: i16) -> Word { (imm5 as u16) & 0b11111 }
        fn N(n: bool) -> Word { (n as u16) << 10 }
        fn Z(z: bool) -> Word { (z as u16) << 9 }
        fn P(p: bool) -> Word { (p as u16) << 8 }
        fn O9(offset9: i16) -> Word { (offset9 as u16) & 0b111111111 }
        fn O11(offset11: i16) -> Word { (1 << 11) | ((offset11 as u16) & 0x7FF) }
        fn Base(base: Reg) -> Word { Sr1(base) }
        fn O6(offset6: i16) -> Word { (offset6 as u16) & 0b111111 }
        fn Sr(sr: Reg) -> Word { Sr1(sr) | 0b111111 }
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
            Ret                       => Op(0b1100)          | Base(Reg::R7)           ,
            Rti                       => Op(0b1000)                                    ,
            St { sr, offset9 }        => Op(0b0011) | Dr(sr)              | O9(offset9),
            Sti { sr, offset9 }       => Op(0b1011) | Dr(sr)              | O9(offset9),
            Str { sr, base, offset6 } => Op(0b0111) | Dr(sr) | Base(base) | O6(offset6),
            Trap { trapvec }          => Op(0b0111)          | Trapvec(trapvec)        ,
        }
    }
}

// TODO: tests for Instruction
// TODO: basic macro
// TODO: add a strict feature

#[cfg(test)]
mod reg_tests {
    use super::Reg::{self, *};
    use core::convert::TryInto;

    #[test]
    fn eq() {
        assert_eq!(R0, R0);
        assert_eq!(R1, R1);
        assert_eq!(R7, R7);

        assert_ne!(R0, R7);
    }

    #[test]
    fn into() {
        let eq = |reg, num| assert_eq!(Into::<u8>::into(reg), num);

        eq(R0, 0);
        eq(R1, 1);
        eq(R2, 2);
        eq(R4, 4);
        eq(R5, 5);
        eq(R5, 5);
        eq(R6, 6);
        eq(R7, 7);
    }

    #[test]
    fn from() {
        let into = |num: u8, reg| assert_eq!(num.try_into(), Ok(reg));
        let err = |num: u8| assert_eq!(TryInto::<Reg>::try_into(num), Err(()));

        into(0, R0);
        into(1, R1);
        into(2, R2);
        into(3, R3);
        into(4, R4);
        into(5, R5);
        into(6, R6);
        into(7, R7);

        err(8);
        err(9);
    }
}

#[cfg(test)]
mod priority_level_tests {
    use super::PriorityLevel::{self, *};
    use core::convert::TryInto;

    #[test]
    fn eq() {
        assert_eq!(PL0, PL0);
        assert_eq!(PL1, PL1);
        assert_eq!(PL7, PL7);

        assert_ne!(PL0, PL7);
    }

    #[test]
    fn into() {
        let eq = |reg, num| assert_eq!(Into::<u8>::into(reg), num);

        eq(PL0, 0);
        eq(PL1, 1);
        eq(PL2, 2);
        eq(PL4, 4);
        eq(PL5, 5);
        eq(PL5, 5);
        eq(PL6, 6);
        eq(PL7, 7);
    }

    #[test]
    fn from() {
        let into = |num: u8, reg| assert_eq!(num.try_into(), Ok(reg));
        let err = |num: u8| assert_eq!(TryInto::<PriorityLevel>::try_into(num), Err(()));

        into(0, PL0);
        into(1, PL1);
        into(2, PL2);
        into(3, PL3);
        into(4, PL4);
        into(5, PL5);
        into(6, PL6);
        into(7, PL7);

        err(8);
        err(9);
    }

    #[test]
    fn ord() {
        assert_eq!(PL0, PL0);

        // PL0 is less than PL1, PL2, ...
        // PL1 is less than PL2, PL3, ...
        // ...
        // PL7 is less than ()
        for n in 0..(PriorityLevel::NUM_LEVELS - 1) {
            let mut iter = PriorityLevel::LEVELS.iter().skip(n);
            let lower = iter.next().unwrap();

            for higher in iter {
                assert!(higher > lower);
            }
        }

        // PL7 is greater than PL6, PL5, ...
        // PL6 is greater than PL5, PL4, ...
        // ...
        // PL0 is greater than ()
        for n in 0..(PriorityLevel::NUM_LEVELS - 1) {
            let mut iter = PriorityLevel::LEVELS.iter().rev().skip(n);
            let higher = iter.next().unwrap();

            for lower in iter {
                assert!(higher > lower);
            }
        }
    }
}
