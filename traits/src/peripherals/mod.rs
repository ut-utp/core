//! Peripherals! The [`Peripherals` supertrait](peripherals::Peripherals) and the rest of the
//! peripheral and device traits.

pub mod adc;
pub mod clock;
pub mod gpio;
pub mod pwm;
pub mod timers;
pub mod input;
pub mod output;

pub use gpio::Gpio;
pub use adc::Adc;
pub use pwm::Pwm;
pub use timers::Timers;
pub use clock::Clock;
pub use input::Input;
pub use output::Output;

pub mod stubs;

use core::marker::PhantomData;

// pub trait RunInContextRef<Wrapped = Self> {
//     fn with_ref<R, F: FnOnce(&Wrapped) -> R>(&self, func: F) -> R;
// }

// pub trait RunInContextMut<Wrapped>: RunInContextRef<Wrapped>/* + NoInteriorMutability*/ {
//     fn with_mut<R, F: FnOnce(&mut Wrapped) -> R>(&mut self, func: F) -> R;
// }

// pub trait RunInContextInteriorMut<Wrapped>: RunInContextMut<Wrapped> {
//     fn with_mut<R, F: FnOnce(&mut Wrapped) -> R>(&self, func: F) -> R;
// }

pub trait Peripherals<'int>:
    Gpio<'int> + Adc + Pwm + Timers<'int> + Clock + Input<'int> + Output<'int>
{
    fn init(&mut self);
}

pub struct PeripheralSet<'int, G, A, P, T, C, I, O/*, GW, AW, PW, TW, CW, IW, OW*/>
where
    G: Gpio<'int>,
    A: Adc,
    P: Pwm,
    T: Timers<'int>,
    C: Clock,
    I: Input<'int>,
    O: Output<'int>,
    // GW: 'p + DerefOrOwned<G>,
    // AW: 'p + DerefOrOwned<A>,
    // PW: 'p + DerefOrOwned<P>,
    // TW: 'p + DerefOrOwned<T>,
    // CW: 'p + DerefOrOwned<C>,
    // IW: 'p + DerefOrOwned<I>,
    // OW: 'p + DerefOrOwned<O>,
{
    // gpio: GW,
    // adc: AW,
    // pwm: PW,
    // timers: TW,
    // clock: CW,
    // input: IW,
    // output: OW,
    gpio: G,
    adc: A,
    pwm: P,
    timers: T,
    clock: C,
    input: I,
    output: O,
    _marker: PhantomData<&'int ()>,
}

// TODO: is default a supertrait requirement or just an additional bound here
// (as in, if all your things implement default, we'll give you a default
// otherwise no).
impl<'p, G, A, P, T, C, I, O> Default for PeripheralSet<'p, G, A, P, T, C, I, O/*, G, A, P, T, C, I, O*/>
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

impl<'p, G, A, P, T, C, I, O/*, GW, AW, PW, TW, CW, IW, OW*/> PeripheralSet<'p, G, A, P, T, C, I, O/*, GW, AW, PW, TW, CW, IW, OW*/>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
    // GW: 'p + DerefOrOwned<G>,
    // AW: 'p + DerefOrOwned<A>,
    // PW: 'p + DerefOrOwned<P>,
    // TW: 'p + DerefOrOwned<T>,
    // CW: 'p + DerefOrOwned<C>,
    // IW: 'p + DerefOrOwned<I>,
    // OW: 'p + DerefOrOwned<O>,
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

    pub fn get_adc(&mut self) -> &mut A {
        &mut self.adc
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

// enum WrapperType {
//     Dual,
//     Mutless,
// }

// // TODO: type to lock on each access that implements
// trait RunInContext<Wrapped> {
//     const TYPE: WrapperType;

//     fn with_ref<R, F: FnOnce(&Wrapped) -> R>(&self, func: F) -> R;
//     fn with_mut<R, F: FnOnce(&mut Wrapped) -> R>(&mut self, func: F) -> R;
// }

// impl<T, P: AsRef<T> + AsMut<T>> RunInContext<T> for P {
//     const TYPE: WrapperType = WrapperType::Dual;

//     fn with_ref<R, F: FnOnce(&T) -> R>(&self, func: F) -> R { func(self.as_ref()) }
//     fn with_mut<R, F: FnOnce(&mut T) -> R>(&mut self, func: F) -> R { func(self.as_mut()) }
// }

// impl<T, I: RunInContext<T>> RunInContext<T> for Arc<I> {
//     fn with_ref<R, F: FnOnce(&T) -> R>(&self, func: F) -> R {
//         func()
//     }

//     fn with_mut<R, F: FnOnce(&T) -> R>(&mut self, func: F) -> F {

//     }
// }

// struct LockOnAccess<T>(T);

// struct BorrowOnAccess<T>(T);

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_trait {
    ($nom:ident, $(#[$attr:meta])* pub trait $trait:ident $(<$lifetime:lifetime>)? $(: $bound:ident )? { $($rest:tt)* }) => {
        $(#[$attr])*
        pub trait $trait $(<$lifetime>)? where Self: $($bound)? { $($rest)* }

        // $crate::deref_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $($rest)* });
        // $crate::borrow_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $($rest)* });
        $crate::peripheral_set_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $crate::func_sig!($nom, $($rest)*); });
        // $crate::peripheral_deref_set_impl!($trait$(<$lifetime>)? $(| $lifetime |)?, { $crate::func_sig!($nom, $($rest)*); });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! deref_impl {
    ($trait:path $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        #[allow(unnecessary_qualification)]
        impl<$($lifetime,)? I, T: Default + core::ops::Deref<Target = I> + core::ops::DerefMut> $trait for T
        where
            I: $trait,
        { $crate::func_sig!(+(*), $($rest)*); }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! borrow_impl {
    ($trait:path $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        #[allow(unnecessary_qualification)]
        impl<$($lifetime,)? I, T: Default + core::ops::Deref<Target = I> + core::ops::DerefMut + core::borrow::Borrow<I> + core::borrow::BorrowMut<I>> $trait for T
        where
            I: $trait,
        { $crate::func_sig!(%(borrow, borrow_mut), $($rest)*); }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! peripheral_set_impl {
    ($trait:ty $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
        impl<$($lifetime,)? 'p, G, A, P, T, C, I, O> $trait for $crate::peripherals::PeripheralSet<'p, G, A, P, T, C, I, O/*, G, A, P, T, C, I, O*/>
        where
            $($lifetime: 'p,)?
            G: $crate::peripherals::gpio::Gpio<'p>,
            A: $crate::peripherals::adc::Adc,
            P: $crate::peripherals::pwm::Pwm,
            T: $crate::peripherals::timers::Timers<'p>,
            C: $crate::peripherals::clock::Clock,
            I: $crate::peripherals::input::Input<'p>,
            O: $crate::peripherals::output::Output<'p>,
        { $($rest)* }
    };
}

// #[doc(hidden)]
// #[macro_export]
// macro_rules! peripheral_deref_set_impl {
//     ($trait:ty $(| $lifetime:lifetime |)?, { $($rest:tt)* }) => {
//         impl<$($lifetime,)? 'p, G, A, P, T, C, I, O, GInner, AInner, PInner, TInner, CInner, IInner, OInner> $trait for $crate::peripherals::PeripheralSet<'p, G, A, P, T, C, I, O>
//         where
//             $($lifetime: 'p,)?
//             G: 'p + $crate::peripherals::gpio::Gpio<'p>,
//             A: 'p + $crate::peripherals::adc::Adc,
//             P: 'p + $crate::peripherals::pwm::Pwm,
//             T: 'p + $crate::peripherals::timers::Timers<'p>,
//             C: 'p + $crate::peripherals::clock::Clock,
//             I: 'p + $crate::peripherals::input::Input<'p>,
//             O: 'p + $crate::peripherals::output::Output<'p>,
//             GInner: 'p + Deref<
//         { $($rest)* }
//     };
// }

#[doc(hidden)]
#[macro_export]
macro_rules! func_sig {
    // [No block + Ret] Our ideal form: specified return type, no block:
    // (none)
    ($(+($indir:tt))? $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident($($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name($($idents : $types),*) -> $ret { compile_error!("trait functions not supported yet!") }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_im())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(mut self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_mut())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(&self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_im())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty; $($rest:tt)*) => {
        #[inline] fn $fn_name(&mut self, $($idents : $types),*) -> $ret { ($($indir$indir)?self)$(.$nom)?$(.$i_mut())?.$fn_name($($idents),*) }
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, $($rest)*);
    };


    // [Block + Ret] Ditch blocks if you've got them:
    // (none)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident($($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name($($idents : $types),*) -> $ret; $($rest)*); };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($self:expr,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(mut self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&self, $($idents : $types),*) -> $ret; $($rest)*); };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) -> $ret:ty $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&mut self, $($idents : $types),*) -> $ret; $($rest)*); };


    // [No Block + No Ret] Add in return types if they're not specified:
    // (none)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident($($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name($($idents : $types),*) -> (); $($rest)*); };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(self, $($idents : $types),*) -> (); $($rest)*); };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(mut self, $($idents : $types),*) -> (); $($rest)*); };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&self, $($idents : $types),*) -> (); $($rest)*); };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*); $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&mut self, $($idents : $types),*) -> (); $($rest)*); };


    // [Block + No Ret] Strip blocks + add in return types:
    // (none)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident( $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name($($idents : $types),*) -> (); $($rest)*); };
    // (self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(self, $($idents : $types),*) -> (); $($rest)*); };
    // (mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(mut self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(mut self, $($idents : $types),*) -> (); $($rest)*); };
    // (&self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&self $($idents : $types),*) -> (); $($rest)*); };
    // (&mut self)
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, $(#[$attr:meta])* fn $fn_name:ident(&mut self$(,)? $($idents:ident : $types:ty),*) $block:block $($rest:tt)*) => {
        $crate::func_sig!($(+($indir))? $(%($i_im, $i_mut))? $($nom)?, fn $fn_name(&mut self, $($idents : $types),*) -> (); $($rest)*); };


    // And, finally, the end:
    ($(+($indir:tt))?  $(%($i_im:ident, $i_mut:ident))? $($nom:ident)?, ) => {};
}

impl<'p, G, A, P, T, C, I, O> Peripherals<'p> for PeripheralSet<'p, G, A, P, T, C, I, O/*, G, A, P, T, C, I, O*/>
where
    G: Gpio<'p>,
    A: Adc,
    P: Pwm,
    T: Timers<'p>,
    C: Clock,
    I: Input<'p>,
    O: Output<'p>,
{
    fn init(&mut self) {}
}

use crate::control::{Snapshot, SnapshotError};

impl<'p, G, A, P, T, C, I, O> Snapshot for PeripheralSet<'p, G, A, P, T, C, I, O>
where
    G: Snapshot + Gpio<'p>,
    A: Snapshot + Adc,
    P: Snapshot + Pwm,
    T: Snapshot + Timers<'p>,
    C: Snapshot + Clock,
    I: Snapshot + Input<'p>,
    O: Snapshot + Output<'p>,

    // This shouldn't be needed since, in order to impl Snapshot your Err type has to
    // implement Into<SnapshotError>.
    SnapshotError: From<<G as Snapshot>::Err>,
    SnapshotError: From<<A as Snapshot>::Err>,
    SnapshotError: From<<P as Snapshot>::Err>,
    SnapshotError: From<<T as Snapshot>::Err>,
    SnapshotError: From<<C as Snapshot>::Err>,
    SnapshotError: From<<I as Snapshot>::Err>,
    SnapshotError: From<<O as Snapshot>::Err>,
{
    type Snap = (
        <G as Snapshot>::Snap,
        <A as Snapshot>::Snap,
        <P as Snapshot>::Snap,
        <T as Snapshot>::Snap,
        <C as Snapshot>::Snap,
        <I as Snapshot>::Snap,
        <O as Snapshot>::Snap,
    );

    type Err = SnapshotError; // TODO: report which thing failed? make it part of the SnapshotError type?

    fn record(&self) -> Result<Self::Snap, Self::Err> {
        Ok((
            self.gpio.record()?,
            self.adc.record()?,
            self.pwm.record()?,
            self.timers.record()?,
            self.clock.record()?,
            self.input.record()?,
            self.output.record()?,
        ))
    }

    fn restore(&mut self, snap: Self::Snap) -> Result<(), Self::Err> {
        let (g, a, p, t, c, i, o) = snap;

        self.gpio.restore(g)?;
        self.adc.restore(a)?;
        self.pwm.restore(p)?;
        self.timers.restore(t)?;
        self.clock.restore(c)?;
        self.input.restore(i)?;
        self.output.restore(o)?;

        Ok(())
    }
}
