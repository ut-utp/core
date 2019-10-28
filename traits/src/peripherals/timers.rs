//! [`Timers` trait](Timers) and related types.

use crate::peripheral_trait;
use lc3_isa::Word;

// TODO: Add Errors
// Timer periods: [0, core::u16::MAX)

#[derive(Copy, Clone)]
pub enum Timer { T0, T1 }
pub const NUM_TIMERS: u8 = 2;
pub type TimerArr<T> = [T; NUM_TIMERS as usize];
pub const TIMERS: TimerArr<Timer> = { use Timer::*; [T0, T1] };

impl From<Timer> for usize {
    fn from(timer: Timer) -> usize {
        use Timer::*;
        match timer {
            T0 => 0,
            T1 => 1,
        }
    }
}

#[derive(Copy, Clone)]
pub enum TimerState {
    Repeated,
    SingleShot,
    Disabled,
}

peripheral_trait! {timers,
pub trait Timers<'a>: Default {
    fn set_state(&mut self, timer: Timer, state: TimerState) -> Result<(), ()>;
    fn get_state(&self, timer: Timer) -> Option<TimerState>;

    fn set_period(&mut self, timer: Timer, milliseconds: Word);
    fn get_period(&self, timer: Timer) -> Option<Word>;

    fn register_interrupt(&mut self, timer: Timer, func: &'a (dyn FnMut(Timer) + Send)) -> Result<(), ()>;
}}
