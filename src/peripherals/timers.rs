//! [`Timers` trait](Timers) and related types.

use crate::Word;

// TODO: Switch to enum for pins
// TODO: Add Errors
// Timer periods: [0, core::u16::MAX)
const NUM_TIMERS: u8 = 2; // T0 - T1
pub enum TimerState {
    Repeated,
    SingleShot,
    Disabled,
}
pub trait Timers {
    fn set_state(&mut self, num: u8, state: TimerState) -> Result<(), ()>;
    fn get_state(&mut self, num: u8) -> Option<TimerState>;

    fn set_period(&mut self, num: u8, milliseconds: Word);
    fn get_period(&mut self, num: u8) -> Option<Word>;

    fn register_interrupt(&mut self, num: u8, func: impl FnMut(u8)) -> Result<(), ()>;
}
