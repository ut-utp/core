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
use core::marker::PhantomData;
use gpio::Gpio;
use pwm::Pwm;
use timers::Timers;

use input::Input;
use output::Output;

pub trait Peripherals<'p>:
    Gpio<'p> + Adc + Pwm + Timers<'p> + Clock + Input<'p> + Output<'p>
{
    fn init(&mut self);
}

pub struct PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
{
    gpio: G,
    adc: A,
    pwm: P,
    timers: T,
    clock: C,
    input: I,
    output: O,
    _marker: PhantomData<&'p ()>,
}

// TODO: is default a supertrait requirement or just an additional bound here
// (as in, if all your things implement default, we'll give you a default
// otherwise no).
impl<'p, G, A, P, T, C, I, O> Default for PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
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
            _marker: PhantomData,
        }
    }
}

impl<'p, G, A, P, T, C, I, O> PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
{
    pub fn new(gpio: G, adc: A, pwm: P, timers: T, clock: C, input: I, output: O) -> Self {
        Self {
            gpio,
            adc,
            pwm,
            timers,
            clock,
            input,
            output,
            _marker: PhantomData,
        }
    }

    pub fn get_gpio(&self) -> &G {
        &self.gpio
    }

    pub fn get_adc(&self) -> &A {
        &self.adc
    }

    pub fn get_pwm(&self) -> &P {
        &self.pwm
    }

    pub fn get_timers(&self) -> &T {
        &self.timers
    }

    pub fn get_clock(&self) -> &C {
        &self.clock
    }

    pub fn get_input(&self) -> &I {
        &self.input
    }

    pub fn get_output(&self) -> &O {
        &self.output
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_trait {
    ($nom:ident, $(#[doc = $doc:expr])* pub trait $trait:ident $(<$lifetime:lifetime>)? $(: $bound:ident )? { $($rest:tt)* }) => {
        $(#[doc = $doc])*
        pub trait $trait $(<$lifetime>)? where Self: $($bound)? { $($rest)* }

        $crate::peripheral_set_impl!($trait<$($lifetime)?> $(| $lifetime |)?, { $crate::func_sig!($nom, $($rest)*); });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_set_impl {
    ($trait:ty $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        impl<$($lifetime,)? 'p, G, A, P, T, C, I, O> $trait for $crate::peripherals::PeripheralSet<'p, G, A, P, T, C, I, O>
        where
            $($lifetime: 'p,)?
            G: 'p + $crate::peripherals::gpio::Gpio<'p>,
            A: 'p + $crate::peripherals::adc::Adc,
            P: 'p + $crate::peripherals::pwm::Pwm,
            T: 'p + $crate::peripherals::timers::Timers<'p>,
            C: 'p + $crate::peripherals::clock::Clock,
            I: 'p + $crate::peripherals::input::Input<'p>,
            O: 'p + $crate::peripherals::output::Output<'p>,
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

impl<'p, G, A, P, T, C, I, O> Peripherals<'p> for PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: 'p + Gpio<'p>,
    A: 'p + Adc,
    P: 'p + Pwm,
    T: 'p + Timers<'p>,
    C: 'p + Clock,
    I: 'p + Input<'p>,
    O: 'p + Output<'p>,
{
    fn init(&mut self) {}
}
