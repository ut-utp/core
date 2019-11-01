//! [`Timers` trait](Timers) and related types.

use crate::peripheral_trait;
use core::ops::{Deref, Index, IndexMut};

use lc3_isa::Word;

// TODO: Add Errors
// Timer periods: [0, core::u16::MAX)

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimerId {
    T0,
    T1,
}

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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimerState {
    Repeated,
    SingleShot,
    Disabled,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimerMiscError;

pub type TimerHandler<'a> = &'a (dyn Fn(TimerId) + Sync);

pub type TimerStateMismatch = (TimerId, TimerState);

peripheral_trait! {timers,
pub trait Timers<'a>: Default {
    fn set_state(&mut self, timer: TimerId, state: TimerState) -> Result<(), TimerMiscError>;  // Should this be infallible (TODO)
    fn get_state(&self, timer: TimerId) -> TimerState;
    fn get_states(&self) -> TimerArr<TimerState> {
        let mut states = TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| states[*t] = self.get_state(*t));

        states
    }

    // TODO: setting the period on an already running timer resets the timer, right?
    // TODO: period of zero? disabled, right?
    fn set_period(&mut self, timer: TimerId, ms: Word) -> Result<(), TimerMiscError>;  // Should this be infallible (TODO)
    fn get_period(&self, timer: TimerId) -> Word;
    fn get_periods(&self) -> TimerArr<Word> {
        let mut periods = TimerArr([0u16; TimerId::NUM_TIMERS]);

        TIMERS
            .iter()
            .for_each(|t| periods[*t] = self.get_period(*t));

        periods
    }


    fn register_interrupt(
        &mut self,
        timer: TimerId,
        handler: TimerHandler<'a>
    ) -> Result<(), TimerMiscError>; // Should this be infallible (TODO)
}}

// TODO: Into Error stuff (see Gpio)

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<'a, T: Timers<'a>> Timers<'a> for Arc<RwLock<T>> {
        fn set_state(&mut self, timer: TimerId, state: TimerState) -> Result<(), TimerMiscError> { // TODO: Infallible?
            RwLock::write(self).unwrap().set_state(timer, state)
        }

        fn get_state(&self, timer: TimerId) -> TimerState {
            RwLock::read(self).unwrap().get_state(timer)
        }

        fn set_period(&mut self, timer: TimerId, ms: Word) -> Result<(), TimerMiscError> { // TODO: Infallible?
            RwLock::write(self).unwrap().set_period(timer, ms)
        }

        fn get_period(&self, timer: TimerId) -> Word {
            RwLock::read(self).unwrap().get_period(timer)
        }

        fn register_interrupt(&mut self, timer: TimerId, handler: TimerHandler<'a>) -> Result<(), TimerMiscError> {
            RwLock::write(self).unwrap().register_interrupt(timer, handler)
        }
    }

}
