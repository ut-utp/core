//! [`Timers` trait](Timers) and related types.

use crate::peripheral_trait;

use lc3_isa::Word;
use lc3_macros::DisplayUsingDebug;

use core::ops::{Deref, Index, IndexMut};
use core::sync::atomic::AtomicBool;
use core::num::NonZeroU16;

use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[derive(DisplayUsingDebug)]
pub enum TimerId { T0, T1, }

impl TimerId {
    pub const NUM_TIMERS: usize = 2;
}

impl From<TimerId> for usize {
    fn from(timer: TimerId) -> usize {
        use TimerId::*;
        match timer {
            T0 => 0,
            T1 => 1,
        }
    }
}

pub const TIMERS: TimerArr<TimerId> = {
    use TimerId::*;
    TimerArr([T0, T1])
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimerArr<T>(pub [T; TimerId::NUM_TIMERS]);

// Once const fn is more stable:
// impl<T: Copy> TimerArr<T> {
//     const fn new(val: T) -> Self {
//         Self([val; TimerId::NUM_TIMERS])
//     }
// }

impl<T> Deref for TimerArr<T> {
    type Target = [T; TimerId::NUM_TIMERS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Index<TimerId> for TimerArr<T> {
    type Output = T;

    fn index(&self, id: TimerId) -> &Self::Output {
        &self.0[usize::from(id)]
    }
}

impl<T> IndexMut<TimerId> for TimerArr<T> {
    fn index_mut(&mut self, id: TimerId) -> &mut Self::Output {
        &mut self.0[usize::from(id)]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimerMode {
    Repeated,
    SingleShot,
}

pub type Period = NonZeroU16;
sa::assert_eq_size!(Period, Word); // Make sure Period is Word sized.

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimerState {
    Disabled,
    WithPeriod(Period)
}

peripheral_trait! {timers,
/// A Timer peripheral for an LC-3 simulator.
///
/// Used for scheduling actions (i.e. LC-3 routines). The peripheral consists of
/// multiple timers which can be independently set to trigger interrupts at a
/// specified length of time in the future. Alternatively, a timer can be set to
/// trigger an interrupt periodically, with a specified length of time between
/// triggers. To specify the action to take when the timer triggers an
/// interrupt, users can write interrupt service routines as part of their LC-3
/// programs.
///
/// # Definition
///
/// Each timer can be set to one of two [`modes`](TimerMode) of operation and
/// two main [`states`](TimerState) representing either the time period the
/// timer waits or the lack thereof.
///
/// After the period elapses, the timer will begin showing that an interrupt
/// occurred.
///
/// ## Interrupts
///
/// To allow the LC-3 to continue running simultaneously, the timers must show
/// they are done by triggering interrupts in the machine. This interface
/// provides no dedicated method meant for querying if a timer has completed a
/// full period. Instead, it provides an interrupt interface to signal to the
/// simulator to trigger interrupts.
///
/// Before anything else can work, a set of interrupt flags
/// [must be registered](Timers::register_interrupt_flags) with the peripheral.
/// These are to provide safe shared state between the simulator's process and
/// the hardware's interrupt-handling processes. Until the flags are registered,
/// the rest of the peripheral will not trigger interrupts. We recommend that
/// operations that require the flags simply panic if they are not present.
///
/// All interrupt flags must initially be `false`, indicating that no interrupts
/// have occurred.
///
/// Getting [whether an interrupt occurred](Timers::interrupt_occurred) returns
/// `true` from when a timer's period has elapsed until that timer's interrupt
/// [flag is reset](Timers::reset_interrupt_flag). We recommend implementing
/// this behavior by simply returning the value of the flag to show when an
/// interrupt occurred, and setting the flag to `true` when appropriate.
///
/// The [last part of the interrupt interface](Timers::interrupts_enabled) shows
/// whether interrupts are enabled for a given timer. If a timer is not in a
/// disabled state (and flags are registered), then its interrupts are enabled.
/// This method is used to show the simulator when it is necessary to check
/// whether interrupts occur on this peripheral. Note this function has a
/// default implementation that does literally the above (returns `true` when a
/// pin's state indicates that it is not [`Disabled`]. This should be suitable
/// for most implementors.
///
/// ## Modes
///
/// Each timer initially returns [`SingleShot`](TimerMode::SingleShot) when the
/// mode is read. The mode returned for a timer does not change until the mode
/// is set to a different one.
///
/// After its mode is set, a timer will set its state to
/// [disabled](TimerState::Disabled) and any interrupts that were meant to occur
/// in the future (as explained below) never occur. Crucially, a timer will
/// disable itself **even if the new mode it is set to matches its current
/// mode**. This is meant to avoid complications or confusion that may arise
/// when changing a timer's mode at the middle or close to the end of its
/// period.
///
/// There are two modes:
/// - [`SingleShot`]: after being set to this mode, when the timer is set to
///   [a state with a period](TimerState::WithPeriod), it must show that an
///   interrupt has occurred as soon as that period of time has elapsed from
///   that point, and then set itself to a
///   [disabled state](TimerState::Disabled).
/// - [`Repeated`]: after being set to this mode, when the timer is set to a
///   [state with a period](TimerState::WithPeriod), it must
///   [show that an interrupt has occurred](Timers::interrupt_occurred) every
///   time that period of time elapses from that point.
///    + This is perhaps subtle: the count for a timer in [`Repeated`] starts
///      again **as soon as the count runs out** and not after the interrupt
///      that was raised is actually processed.
///
/// Put simply, when set to [`SingleShot`], a timer will cause **one** (1)
/// interrupt after a specified length of time; when set to [`Repeated`], it
/// will cause interrupts periodically — one after every interval of time of
/// that length.
///
/// ## States
///
/// Each timer initially returns [`Disabled`] when the state is read.
///
/// The state returned for a timer does not change until the state is
/// [set to a different one](Timers::set_state), unless
/// [disabled by a change in the mode](#modes).
///
/// Each timer holds two possible main states. Either the timer
/// [has a period](TimerState::WithPeriod), representing how much time must
/// elapse before an interrupt or the timer is [disabled](TimerState::Disabled),
/// indicating it will not cause any interrupts until it is next set.
///
/// Whenever the period is set (even if it's set to the current period value),
/// the interval count that the timer maintains is reset. For example, if timer
/// [`T0`](TimerId::T0) was configured with in [`SingleShot`] mode with a period
/// of 100ms, 50ms ago and we go and set [`T0`]'s period to 100ms again,
/// [`T0`]'s interrupt won't fire until 100ms from now, even though there _were_
/// only 50ms left on [`T0`]'s count.
///
/// When the state is set to [`Disabled`], no interrupts will occur.
///
/// When the state is set to [have a period](TimerState::WithPeriod), the timer
/// should be set to show an interrupt occurred after that period of time
/// elapses from that point, either [once](TimerMode::SingleShot) or
/// [periodically](TimerMode::Repeated) as described in the section on
/// [Modes](#modes).
///
/// # Reasoning
///
/// We provide the [`Timers`](Timers) peripheral to enable LC-3 users to
/// schedule or delay routines. By setting a timer, a user can, for example,
/// toggle a "heartbeat" output which shows their program is running without
/// requiring a main program loop dependent on instruction execution speed.
///
/// A user could also cause their program to pause for a specified time without
/// polling on [`Clock`](lc3_traits::peripherals::Clock) in a tight loop.
///
/// Additionally, though this isn't explicitly a goal of this project, it has
/// not escaped our attention that providing timer peripherals makes it feasible
/// to write a true preemptive multitasking operating system for the LC-3; we
/// believe that offering the _ability_ to do this is part of the contract of a
/// complete and usable set of peripherals and helps us get closer to providing
/// a minimal but realistic pedagogical computer.
///
/// One notable way we differ from many (most?) real hardware timer peripherals
/// is that we offer no way to read the running 'count' of a timer. This is
/// intentional (for the sake of simplicity). We do appreciate that there are
/// use cases that require this (i.e. measuring the time between events or the
/// duration of an event) and for those we offer the
/// [`Clock` peripheral](lc3_traits::peripherals::Clock).
///
/// [`SingleShot`]: TimerMode::SingleShot
/// [`Repeated`]: TimerMode::Repeated
/// [`Disabled`]: TimerState::Disabled
///
pub trait Timers<'a>: Default {
    fn set_mode(&mut self, timer: TimerId, mode: TimerMode);
    fn get_mode(&self, timer: TimerId) -> TimerMode;
    fn get_modes(&self) -> TimerArr<TimerMode> {
        let mut modes = TimerArr([TimerMode::SingleShot; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| modes[*t] = self.get_mode(*t));

        modes
    }

    fn set_state(&mut self, timer: TimerId, state: TimerState);
    fn get_state(&self, timer: TimerId) -> TimerState;
    fn get_states(&self) -> TimerArr<TimerState> {
        let mut states = TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| states[*t] = self.get_state(*t));

        states
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>);
    fn interrupt_occurred(&self, timer: TimerId) -> bool;
    fn reset_interrupt_flag(&mut self, timer: TimerId);
    fn interrupts_enabled(&self, timer: TimerId) -> bool {
        matches!(self.get_state(timer), TimerState::WithPeriod(_))
    }
}}

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<'a, T: Timers<'a>> Timers<'a> for Arc<RwLock<T>> {
        fn set_mode(&mut self, timer: TimerId, mode: TimerMode) {
            RwLock::write(self).unwrap().set_mode(timer, mode);
        }

        fn get_mode(&self, timer: TimerId) -> TimerMode {
            RwLock::read(self).unwrap().get_mode(timer)
        }

        fn set_state(&mut self, timer: TimerId, state: TimerState) {
            RwLock::write(self).unwrap().set_state(timer, state);
        }

        fn get_state(&self, timer: TimerId) -> TimerState {
            RwLock::read(self).unwrap().get_state(timer)
        }

        fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {
            RwLock::write(self).unwrap().register_interrupt_flags(flags)
        }

        fn interrupt_occurred(&self, timer: TimerId) -> bool {
            RwLock::read(self).unwrap().interrupt_occurred(timer)
        }

        fn reset_interrupt_flag(&mut self, timer: TimerId) {
            RwLock::write(self).unwrap().reset_interrupt_flag(timer)
        }

        fn interrupts_enabled(&self, timer: TimerId) -> bool {
            RwLock::read(self).unwrap().interrupts_enabled(timer)
        }

    }

}
