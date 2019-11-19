//! Constants. (TODO)

use crate::interp::InstructionInterpreterPeripheralAccess;
use core::ops::Deref;
use lc3_isa::{Addr, Bits, SignedWord, Word, MCR as MCR_ADDRESS, PSR as PSR_ADDRESS, WORD_MAX_VAL};
use lc3_traits::peripherals::Peripherals;

use crate::interp::{Acv, InstructionInterpreter, WriteAttempt};

pub trait MemMapped: Deref<Target = Word> + Sized {
    const ADDR: Addr;

    fn with_value(value: Word) -> Self;

    fn from<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> Result<Self, Acv>
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        // Checked access by default:
        Ok(Self::with_value(interp.get_word(Self::ADDR)?))
    }

    fn set<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, value: Word) -> WriteAttempt
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        // Checked access by default:
        interp.set_word(Self::ADDR, value)
    }

    fn update<I: InstructionInterpreterPeripheralAccess>(
        interp: &mut I,
        func: impl FnOnce(Self) -> Word,
    ) -> WriteAttempt
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        Self::set(interp, func(Self::from(interp)?))
    }

    #[doc(hidden)]
    fn write_current_value<I: InstructionInterpreterPeripheralAccess>(
        &self,
        interp: &mut I,
    ) -> WriteAttempt
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        Self::set(interp, **self)
    }
}

// Don't implement this manually; you could mess up. (only implement this if
// you've overriden the default impls for from and set in the trait above).
//
// Use the macro below instead.
pub trait MemMappedSpecial: MemMapped {
    // Infallible.
    fn from_special<I: InstructionInterpreterPeripheralAccess>(interp: &I) -> Self
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        Self::from(interp).unwrap()
    }

    // Also infallible.
    fn set_special<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, value: Word)
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        Self::set(interp, value).unwrap()
    }
}

// struct KBSR(Word);

// impl Deref for KBSR {
//     type Target = Word;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl MemMapped for KBSR {
//     const ADDR: Addr = 0xFE00;

//     fn with_value(value: Word) -> Self {
//         Self(value)
//     }
// }

macro_rules! mem_mapped {
    ($name:ident, $address:expr, $comment:literal) => {
        #[doc=$comment]
        mem_mapped!(-; $name, $address, $comment);
    };

    (special: $name:ident, $address:expr, $comment:literal) => {
        #[doc=$comment]
        #[doc="\nDoes not produce access control violations (ACVs) when accessed!"]
        mem_mapped!(+; $name, $address, $comment);

        impl MemMappedSpecial for $name { }
    };

    ($special:tt; $name:ident, $address:expr, $comment:literal) => {
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $name(Word);

        impl Deref for $name {
            type Target = Word;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl MemMapped for $name {
            const ADDR: Addr = $address;

            fn with_value(value: Word) -> Self {
                Self(value)
            }

            _mem_mapped_special_access!($special);
        }
    }
}

macro_rules! _mem_mapped_special_access {
    (+) => {
        fn from<I: InstructionInterpreter>(interp: &I) -> Result<Self, Acv> {
            // Special unchecked access!
            Ok(Self::with_value(interp.get_word_unchecked(Self::ADDR)))
        }

        fn set<I: InstructionInterpreter>(interp: &mut I, value: Word) -> WriteAttempt {
            // Special unchecked access!
            interp.set_word_unchecked(Self::ADDR, value);
            Ok(())
        }
    };
    (-) => {};
}

// struct KBSR(Word);

// impl Deref for KBSR {
//     type Target = Word;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl MemMapped for KBSR {
//     const ADDR: Addr = 0xFE00;

//     fn with_value(value: Word) -> Self {
//         Self(value)
//     }
// }

mem_mapped!(KBSR, 0xFE00, "Keyboard Status Register.");
mem_mapped!(KBDR, 0xFE02, "Keyboard Data Register.");

// impl KBSR {
//     pub fn
// }

mem_mapped!(DSR, 0xFE04, "Display Status Register.");
mem_mapped!(DDR, 0xFE06, "Display Data Register.");

macro_rules! gpio_mem_mapped {
    ($pin:expr, $pin_name:literal, $cr:ident, $dr:ident, $addr:expr) => {
        #[doc=$pin_name]
        #[doc="GPIO Pin Control Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $cr(Word);

        impl Deref for $cr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $cr {
            const ADDR: Addr = $addr;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: InstructionInterpreterPeripheralAccess> (interp: &I) -> Result<Self, Acv>
            where for <'a> <I as Deref>::Target: Peripherals<'a> {
                let state = Gpio::get_state(interp.get_peripherals(), $pin);

                use lc3_traits::peripherals::gpio::GpioState::*;
                let word: Word = match state {
                    Disabled => 0,
                    Output => 1,
                    Input => 2,
                    Interrupt => 3,
                };

                Ok(Self::with_value(word))
            }

            fn set<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, value: Word) -> WriteAttempt
            where for <'a> <I as Deref>::Target: Peripherals<'a> {
                use lc3_traits::peripherals::gpio::GpioState::*;
                let state = match value.bits(0..2) {
                    0 => Disabled,
                    1 => Output,
                    2 => Input,
                    3 => Interrupt,
                    _ => unreachable!()
                };

                Gpio::set_state(interp.get_peripherals_mut(), $pin, state).unwrap(); // TODO: do something different on error?

                Ok(())
            }
        }

        #[doc=$pin_name]
        #[doc="GPIO Pin Data Register"] // TODO: format correctly
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $dr(Word);

        impl Deref for $dr {
            type Target = Word;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl MemMapped for $dr {
            const ADDR: Addr = $addr + 1;

            fn with_value(value: Word) -> Self { Self(value) }

            fn from<I: InstructionInterpreterPeripheralAccess> (interp: &I) -> Result<Self, Acv> // TODO: change all these to some other kind of error since we already check for ACVs in read_word, etc.
            where for <'a> <I as Deref>::Target: Peripherals<'a> {
                let word = Gpio::read(interp.get_peripherals(), $pin).map(|b| b as Word).unwrap_or(2); // TODO: document and/or change the 'error' value

                Ok(Self::with_value(word))
            }

            fn set<I: InstructionInterpreterPeripheralAccess>(interp: &mut I, value: Word) -> WriteAttempt
            where for <'a> <I as Deref>::Target: Peripherals<'a> {
                let bit: bool = value.bit(0);
                Gpio::write(interp.get_peripherals_mut(), $pin, bit); // TODO: do something on failure

                Ok(())
            }
        }
    };
}

use lc3_traits::peripherals::gpio::{Gpio, GpioPin::*};

gpio_mem_mapped!(G0, "G0", G0CR, G0DR, 0xFE07);
gpio_mem_mapped!(G1, "G1", G1CR, G1DR, 0xFE09);
gpio_mem_mapped!(G2, "G2", G2CR, G2DR, 0xFE0B);
gpio_mem_mapped!(G3, "G3", G3CR, G3DR, 0xFE0D);
gpio_mem_mapped!(G4, "G4", G4CR, G4DR, 0xFE0F);
gpio_mem_mapped!(G5, "G5", G5CR, G5DR, 0xFE11);
gpio_mem_mapped!(G6, "G6", G6CR, G6DR, 0xFE13);
gpio_mem_mapped!(G7, "G7", G7CR, G7DR, 0xFE15);

mem_mapped!(special: BSP, 0xFFFA, "Backup Stack Pointer.");

mem_mapped!(special: PSR, PSR_ADDRESS, "Program Status Register.");

impl PSR {
    pub fn get_priority(&self) -> u8 {
        self.u8(8..10)
    }

    pub fn set_priority<I: InstructionInterpreterPeripheralAccess>(
        &mut self,
        interp: &mut I,
        priority: u8,
    ) where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        self.0 = (self.0 & (!WORD_MAX_VAL.word(8..10))) | (priority as Word);

        // Don't return a `WriteAttempt` since PSR accesses don't produce ACVs (and are hence infallible).
        self.write_current_value(interp).unwrap();
    }

    pub fn in_user_mode(&self) -> bool {
        self.bit(15)
    }
    pub fn in_privileged_mode(&self) -> bool {
        !self.in_user_mode()
    }

    fn set_mode<I: InstructionInterpreterPeripheralAccess>(
        &mut self,
        interp: &mut I,
        user_mode: bool,
    ) where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        self.0 = self.0.u16(0..14) | (Into::<Word>::into(user_mode) << 15);

        // Don't return a `WriteAttempt` since PSR accesses are infallible.
        self.write_current_value(interp).unwrap()
    }

    pub fn to_user_mode<I: InstructionInterpreterPeripheralAccess>(&mut self, interp: &mut I)
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        self.set_mode(interp, true)
    }

    pub fn to_privileged_mode<I: InstructionInterpreterPeripheralAccess>(&mut self, interp: &mut I)
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        self.set_mode(interp, false)
    }

    pub fn n(&self) -> bool {
        self.bit(2)
    }
    pub fn z(&self) -> bool {
        self.bit(1)
    }
    pub fn p(&self) -> bool {
        self.bit(0)
    }
    pub fn get_cc(&self) -> (bool, bool, bool) {
        (self.n(), self.z(), self.p())
    }

    pub fn set_cc<I: InstructionInterpreterPeripheralAccess>(&mut self, interp: &mut I, word: Word)
    where
        for<'a> <I as Deref>::Target: Peripherals<'a>,
    {
        let word = word as SignedWord;

        // checking for n is easy once we've got a `SignedWord`.
        let n: bool = word.is_negative();

        // z is easy enough to check for:
        let z: bool = word == 0;

        // if we're not negative or zero, we're positive:
        let p: bool = !(n | z);

        fn bit_to_word(bit: bool, left_shift: u32) -> u16 {
            (if bit { 1 } else { 0 }) << left_shift
        }

        let b = bit_to_word;

        self.0 = (self.0 & !(WORD_MAX_VAL.word(0..2))) | b(n, 2) | b(z, 1) | b(p, 0);

        // Don't return a `WriteAttempt` since PSR accesses are infallible.
        self.write_current_value(interp).unwrap();
    }
}

mem_mapped!(special: MCR, MCR_ADDRESS, "Machine Control Register.");

// pub const KBDR: Addr = 0xFE02;

// // pub const KBSR: Addr = 0xFE00;
// pub const KBDR: Addr = 0xFE02;
// pub const DSR: Addr = 0xFE04;
// pub const DDR: Addr = 0xFE06;
// pub const BSP: Addr = 0xFFFA;
