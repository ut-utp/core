//! Format impls. (TODO)

use super::isa::{Instruction, Reg};
use core::fmt::{self, Display};

impl Display for Reg {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Reg::*;
        write!(
            fmt,
            "R{}",
            match self {
                R0 => '0',
                R1 => '1',
                R2 => '2',
                R3 => '3',
                R4 => '4',
                R5 => '5',
                R6 => '6',
                R7 => '7',
            }
        )
    }
}

impl Display for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        match self {
            AddReg { dr, sr1, sr2 } => write!(fmt, "ADD   {}, {}, {}", dr, sr1, sr2),
            AddImm { dr, sr1, imm5 } => write!(fmt, "ADD   {}, {}, #{}", dr, sr1, imm5),
            AndReg { dr, sr1, sr2 } => write!(fmt, "AND   {}, {}, {}", dr, sr1, sr2),
            AndImm { dr, sr1, imm5 } => write!(fmt, "AND   {}, {}, #{}", dr, sr1, imm5),
            Br { n, z, p, offset9 } => {
                let spaces =
                    if *n { 0 } else { 1 } + if *z { 0 } else { 1 } + if *p { 0 } else { 1 };

                write!(fmt, "BR")
                    .and_then(|_| if *n { write!(fmt, "n") } else { Ok(()) })
                    .and_then(|_| if *z { write!(fmt, "z") } else { Ok(()) })
                    .and_then(|_| if *p { write!(fmt, "p") } else { Ok(()) })
                    .and_then(|_| {
                        for _ in 0..=spaces {
                            write!(fmt, " ")?
                        }
                        write!(fmt, "#{}", offset9)
                    })
            }
            Jmp { base } => write!(fmt, "JMP   {}", base),
            Jsr { offset11 } => write!(fmt, "JSR   #{}", offset11),
            Jsrr { base } => write!(fmt, "JSRR  {}", base),
            Ld { dr, offset9 } => write!(fmt, "LD    {}, #{}", dr, offset9),
            Ldi { dr, offset9 } => write!(fmt, "LDI   {}, #{}", dr, offset9),
            Ldr { dr, base, offset6 } => write!(fmt, "LDR   {}, {}, #{}", dr, base, offset6),
            Lea { dr, offset9 } => write!(fmt, "LEA   {}, #{}", dr, offset9),
            Not { dr, sr } => write!(fmt, "NOT   {}, {}", dr, sr),
            Ret => write!(fmt, "RET"),
            Rti => write!(fmt, "RTI"),
            St { sr, offset9 } => write!(fmt, "ST    {}, #{}", sr, offset9),
            Sti { sr, offset9 } => write!(fmt, "STI   {}, #{}", sr, offset9),
            Str { sr, base, offset6 } => write!(fmt, "STR   {}, {}, #{}", sr, base, offset6),
            Trap { trapvec } => write!(fmt, "TRAP  x{:X}", trapvec),
        }
    }
}
