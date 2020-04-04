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
pub enum Mode {
    Repeated,
    SingleShot,
}

pub type Period = NonZeroU16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum State {
    Disabled,
    WithPeriod(Period)
}

/// A Timers peripheral for an LC-3 simulator.
///
/// Used for scheduling actions (i.e. LC-3 routines).
/// The peripheral consists of multiple timers which can be independently set to trigger
/// interrupts at a specified length of time in the future.
/// Alternatively, a timer can be set to trigger an interrupt periodically,
/// with a specified length of time between triggers.
/// To specify the action to take when the timer triggers an interrupt,
/// users can write interrupt service routines as part of their LC-3 programs.
///
/// # Simple Definition
///
/// # General Definition
///
/// # Reasoning
///
/// We provide the Timers peripheral to enable simulator users to schedule or delay routines.
/// By setting a timer, a user can toggle a "heartbeat" output which shows their program is running
/// without requiring a main program loop dependent on instruction execution speed.
/// A user could also cause their program to pause for a specified time without polling on Clock in a tight loop.
///
///
peripheral_trait! {timers,
pub trait Timers<'a>: Default {
    fn set_mode(&mut self, timer: TimerId, mode: Mode);
    fn get_mode(&self, timer: TimerId) -> Mode;
    fn get_modes(&self) -> TimerArr<Mode> {
        let mut modes = TimerArr([Mode::SingleShot; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| modes[*t] = self.get_mode(*t));

        modes
    }

    fn set_state(&mut self, timer: TimerId, state: State);
    fn get_state(&self, timer: TimerId) -> State;
    fn get_states(&self) -> TimerArr<State> {
        let mut states = TimerArr([State::Disabled; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| states[*t] = self.get_state(*t));

        states
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>);
    fn interrupt_occurred(&self, timer: TimerId) -> bool;
    fn reset_interrupt_flag(&mut self, timer: TimerId);
    fn interrupts_enabled(&self, timer: TimerId) -> bool;

}}

// TODO: Into Error stuff (see Gpio)

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<'a, T: Timers<'a>> Timers<'a> for Arc<RwLock<T>> {
        fn set_mode(&mut self, timer: TimerId, mode: Mode) {
            RwLock::write(self).unwrap().set_mode(timer, mode);
        }

        fn get_mode(&self, timer: TimerId) -> Mode {
            RwLock::read(self).unwrap().get_mode(timer)
        }

        fn set_state(&mut self, timer: TimerId, state: State) {
            RwLock::write(self).unwrap().set_state(timer, state);
        }

        fn get_state(&self, timer: TimerId) -> State {
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
