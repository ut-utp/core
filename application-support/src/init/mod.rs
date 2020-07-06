//! Provides common case initialization functions for the first party devices
//! (i.e. the baseline simulator, the UART based rpc controller, etc.).
//!
//! This isn't meant to be all encompassing: particular devices and peripheral
//! + memory impl combinations will have lots of knobs and lots of things that
//! _can_ be configured. However, applications will usually not try to expose
//! these configuration options and will instead only allow users to swtich
//! between a few common predefined setups.
//!
//! The goal of this module is to provide said setups.
//!
//! Others can make their own types that implement the [`Init`] trait (and use
//! the [`BlackBox`] type to store their state) to provide different
//! configurations (simply just different peripheral/memory/interpreter
//! combinations or using entirely custom implementations — everything is
//! supposed to be modular!) for applications to use.
//!
//! The first party applications (the TUI and GUI) use this module to set up and
//! store state for the different configurations that are supported but you, as
//! an application developer, are of course free to circumvent this entirely and
//! to configure the [`Control`] implementation(s) of your choice as you see
//! fit.
//!
//! [`BlackBox`]: `BlackBox`
//! [`Control`]: `lc3_traits::control::Control`
//! [`Init`]: `Init`
//!
//! TODO!

use crate::{
    io_peripherals::{InputSink, OutputSource},
    shim_support::Shims,
};

use lc3_traits::control::Control;

use std::any::Any;

pub mod sim;
pub mod websocket;

pub use sim::*;
pub use websocket::*;

not_wasm! {
    pub mod board;
    pub mod sim_rpc;

    pub use board::*;
    pub use sim_rpc::*;
}

#[derive(Debug)]
pub struct BlackBox {
    inner: Box<dyn Any>,
}

impl BlackBox {
    pub fn new() -> Self {
        Self {
            inner: Box::new(()),
        }
    }
}

impl BlackBox {
    pub fn put<'a, T: 'static>(&'a mut self, data: T) -> &'a mut T {
        let data: Box<dyn Any> = Box::new(data);
        drop(std::mem::replace(&mut self.inner, data));

        self.inner.downcast_mut().unwrap()
    }
}

/// As mentioned in the module docs, this trait does not attempt to cover all
/// the different configuration interfaces that [`Control`] implementors can
/// offer: instead we go after the common case. Put differently, this trait is
/// restrictive and limiting _by design_.
///
/// As such, this trait offers only a zero configuration function and a function
/// that takes a configuration type of the implementor's choosing. More
/// complicated initialization interfaces (i.e. builders, things involving type
/// states) are not supported. Also not supported are configuration rituals that
/// can produce different concrete types (i.e. if a user chooses to, for
/// example, use a [`FileBackedMemoryShim`] instead of a [`MemoryShim`],
/// the resulting concrete type of the [`Interpreter`] that's produced will be
/// different — a [`Init`] impl that wants to support configuration options
/// would need to resort to trait object magicks to provide this functionality
/// while having a consistent `ControlImpl` type below).
///
/// [`Control`]: `lc3_traits::control::Control`
/// [`MemoryShim`]: `lc3_shims::peripherals::MemoryShim`
/// [`FileBackedMemoryShim`]: `lc3_shims::peripherals::FileBackedMemoryShim`
/// [`Interpreter`]: `lc3_baseline_sim::interp::Interpreter`
/// [`Init`]: `Init`
/// [`ControlImpl`]: `Init::ControlImpl`
pub trait Init<'s> {
    type Config: Default; // Once associated type defaults are stable: `= ()`

    type ControlImpl: Control + ?Sized + 's;
    type Input: InputSink + ?Sized + 's;
    type Output: OutputSource + ?Sized + 's;

    fn init(
        b: &'s mut BlackBox,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    ) {
        Self::init_with_config(b, Default::default())
    }

    fn init_with_config(
        b: &'s mut BlackBox,
        config: Self::Config,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    );
}
