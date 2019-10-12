//! Peripherals! The [`Peripherals` supertrait](peripherals::Peripherals) and the rest of the
//! peripheral and device traits.

pub mod adc;
pub mod clock;
pub mod gpio;
pub mod pwm;
pub mod timers;

pub mod input;
pub mod output;

use adc::Adc;
use clock::Clock;
use gpio::Gpio;
use pwm::Pwm;
use timers::Timers;

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
    output: O,
}

// TODO: is default a supertrait requirement or just an additional bound here
// (as in, if all your things implement default, we'll give you a default
// otherwise no).
impl<G, A, P, T, C, I, O> Default for PeripheralSet<G, A, P, T, C, I, O>
where
    G: Gpio,
    A: Adc,
    P: Pwm,
    T: Timers,
    C: Clock,
    I: Input,
    O: Output,
{
    fn default() -> Self {
        Self {
            gpio: G::default(),
            adc: A::default(),
            pwm: P::default(),
            timers: T::default(),
            clock: C::default(),
            input: I::default(),
            output: O::default(),
        }
    }
}
#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_trait {
    ($nom:ident, $(#[doc = $doc:expr])* pub trait $trait:ident $(: $bound:ident )? { $($rest:tt)* }) => {
        $(#[doc = $doc])*
        pub trait $trait where Self: $($bound)? { $($rest)* }

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
    // [No block + Ret] Our ideal form: specified return type, no block:
    // (none)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident($($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name($($idents : $types),*) -> $ret { compile_error!("trait functions not supported yet!") }
        $crate::func_sig!($nom, $($rest)*);
    };
    // (self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(self, $($idents : $types),*) -> $ret { self.$nom.$fn_name($($idents),*) }
        $crate::func_sig!($nom, $($rest)*);
    };
    // (mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(mut self, $($idents : $types),*) -> $ret { self.$nom.$fn_name($($idents),*) }
        $crate::func_sig!($nom, $($rest)*);
    };
    // (&self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(&self, $($idents : $types),*) -> $ret { self.$nom.$fn_name($($idents),*) }
        $crate::func_sig!($nom, $($rest)*);
    };
    // (&mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        fn $fn_name(&mut self, $($idents : $types),*) -> $ret { self.$nom.$fn_name($($idents),*) }
        $crate::func_sig!($nom, $($rest)*);
    };


    // [Block + Ret] Ditch blocks if you've got them:
    // (none)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident($($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name($($idents : $types),*) -> $ret; $($rest)*); };
    // (self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(self, $($self:expr,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(mut self, $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(mut self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (&self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&self $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (&mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&mut self, $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&mut self, $($idents : $types),*) -> $ret; $($rest)*); };


    // [No Block + No Ret] Add in return types if they're not specified:
    // (none)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident($($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name($($idents : $types),*) -> (); $($rest)*); };
    // (self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(self, $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(self, $($idents : $types),*) -> (); $($rest)*); };
    // (mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(mut self, $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(mut self, $($idents : $types),*) -> (); $($rest)*); };
    // (&self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&self, $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&self, $($idents : $types),*) -> (); $($rest)*); };
    // (&mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&mut self, $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&mut self, $($idents : $types),*) -> (); $($rest)*); };


    // [Block + No Ret] Strip blocks + add in return types:
    // (none)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident( $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name($($idents : $types),*) -> (); $($rest)*); };
    // (self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(self, $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(self, $($idents : $types),*) -> (); $($rest)*); };
    // (mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(mut self, $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(mut self, $($idents : $types),*) -> (); $($rest)*); };
    // (&self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&self, $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&self $($idents : $types),*) -> (); $($rest)*); };
    // (&mut self)
    ($nom:ident, $(#[doc = $doc:expr])* fn $fn_name:ident(&mut self, $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($nom, fn $fn_name(&mut self, $($idents : $types),*) -> (); $($rest)*); };


    // And, finally, the end:
    ($nom:ident, ) => {};
}

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
//     // fn init() ->
// }
