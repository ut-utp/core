//! Peripherals! The [`Peripherals` supertrait](trait.Peripherals.html) and the
//! rest of the peripheral and device traits.

pub mod gpio;
pub mod adc;
pub mod pwm;
pub mod timers;
pub mod clock;

pub mod input;
pub mod output;

use gpio::Gpio;
use adc::Adc;
use pwm::Pwm;
use timers::Timers;
use clock::Clock;

use input::Input;
use output::Output;

pub trait Peripherals: Gpio + Adc + Pwm + Timers + Clock + Input + Output {
    fn init() -> Self;
}

pub struct PeripheralSet<G, A, P, T, C, I, O>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
{
    gpio: G,
    adc: A,
    pwm: P,
    timers: T,
    clock: C,
    input: I,
    output: O
}

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_trait {
    ($nom:ident, pub trait $trait:ident { $($rest:tt)* }) => {
        pub trait $trait { $($rest)* }

        $crate::peripheral_set_impl!($trait, { $crate::func_sig!($nom, $($rest)*); });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_set_impl {
    ($trait:ty, { $($rest:tt)* }) => {
        impl<G, A, P, T, C, I, O> $trait for $crate::peripherals::PeripheralSet<G, A, P, T, C, I, O>
        where
            G: $crate::peripherals::gpio::Gpio,
            A: $crate::peripherals::adc::Adc,
            P: $crate::peripherals::pwm::Pwm,
            T: $crate::peripherals::timers::Timers,
            C: $crate::peripherals::clock::Clock,
            I: $crate::peripherals::input::Input,
            O: $crate::peripherals::output::Output,
        { $($rest)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! func_sig {
    ($nom:ident, fn $fn_name:ident(&mut self, $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(&mut self, $($idents : $types),*) -> $ret {
            self.$nom.$fn_name($($idents),*)
        }

        $crate::func_sig!($nom, $($rest)*);
    };
    ($nom:ident, fn $fn_name:ident(mut self, $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(mut self, $($idents : $types),*) -> $ret {
            self.$nom.$fn_name($($idents),*)
        }

        $crate::func_sig!($nom, $($rest)*);
    };
    ($nom:ident, fn $fn_name:ident(&self, $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(&self, $($idents : $types),*) -> $ret {
            self.$nom.$fn_name($($idents),*)
        }

        $crate::func_sig!($nom, $($rest)*);
    };
    ($nom:ident, fn $fn_name:ident(self, $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(self, $($idents : $types),*) -> $ret {
            self.$nom.$fn_name($($idents),*)
        }

        $crate::func_sig!($nom, $($rest)*);
    };

    // If we've been given a default impl somehow, ditch it:
    ($nom:ident, fn $fn_name:ident(&mut self, $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&mut self, $($idents : $types),*) -> $ret; $($rest)*);
    };
    ($nom:ident, fn $fn_name:ident(mut self, $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(mut self, $($idents : $types),*) -> $ret; $($rest)*);
    };
    ($nom:ident, fn $fn_name:ident(&self, $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&self, $($idents : $types),*) -> $ret; $($rest)*);
    };
    ($nom:ident, fn $fn_name:ident(self, $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(self, $($idents : $types),*) -> $ret; $($rest)*);
    };

    // And, finally, the end:
    ($nom:ident, ) => {};
}


// impl<G, A, P, T, C, I, O> Gpio for PeripheralSet<G, A, P, T, C, I, O>
// where
//     G: Gpio,
//     A: Adc,
//     P: Pwm,
//     T: Timers,
//     C: Clock,
//     I: Input,
//     O: Output,
//     {}

// impl<G, A, P, T, C, I, O> Peripherals for PeripheralSet<G, A, P, T, C, I, O>
// where
//     G: Gpio,
//     A: Adc,
//     P: Pwm,
//     T: Timers,
//     C: Clock,
//     I: Input,
//     O: Output,
// {

// }
