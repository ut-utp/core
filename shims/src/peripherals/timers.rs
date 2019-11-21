use lc3_traits::peripherals::timers::{
    TimerArr, TimerId, TimerMiscError, TimerState, TimerStateMismatch, Timers,
};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
use std::cell::Cell;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use core::sync::atomic::AtomicBool;

// The term “Single Shot” signifies a single pulse output of some duration.
pub struct TimersShim<'a> {
    states: TimerArr<TimerState>,
    times: TimerArr<Word>,
    flags: TimerArr<Option<&'a AtomicBool>>,
}

impl Default for TimersShim<'_> {
    fn default() -> Self {
        Self {
            states: TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]),
            times: TimerArr([0u16; TimerId::NUM_TIMERS]), // unlike gpio, interrupts occur on time - not on bit change
            flags: TimerArr([None; TimerId::NUM_TIMERS]),
        }
    }
}

impl TimersShim<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    fn singleshot_timer(&mut self, timer: TimerId) {
        // let state_fixture = Mutex::new(Cell::new(self.times[timer]));

        // thread::spawn(move || {
        //     let mut state_fixture = state_fixture.lock().unwrap();
        //     thread::sleep(Duration::from_millis((*state_fixture).get().unwrap() as u64));
        // });

        unimplemented!()
    }

    fn repeated_timer(&mut self, timer: TimerId) {
        // let state_fixture = Mutex::new(Cell::new(self.times[timer]));
        // let handle = thread::spawn(move || loop {
        //     let mut state_fixture = state_fixture.lock().unwrap();
        //     thread::sleep(Duration::from_millis((*state_fixture).get().unwrap() as u64));
        // });

        unimplemented!()
    }
}

impl Timers<'_> for TimersShim<'_> {
    fn set_state(&mut self, timer: TimerId, state: TimerState) -> Result<(), TimerMiscError> {
        self.states[timer] = state;

        Ok(())
    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        self.states[timer]
    }

    fn set_period(&mut self, timer: TimerId, milliseconds: Word) -> Result<(), TimerMiscError> {
        // thread based
        // self.times[timer] = Some(milliseconds);
        // // let temp = thread::Builder::new(); TODO: add return thread to kill repeated timers...
        // use TimerState::*;
        // match self.states[timer] {
        //     Repeated => self.repeated_timer(timer),
        //     SingleShot => self.singleshot_timer(timer),
        //     Disabled => (),
        // };

        // Ok(())
        unimplemented!()
    }

    fn get_period(&self, timer: TimerId) -> Word {
        self.times[timer]
    }

}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use lc3_traits::peripherals::timers::{Timer::*, Timers};

//     #[test]
//     fn get_disabled() {
//         let shim = TimersShim::new();
//         assert_eq!(shim.get_state(T0).unwrap(), TimerState::Disabled);
//     }

//     #[test]
//      fn get_singleshot() {
//         let mut shim = TimersShim::new();
//         let res = shim.set_state(T0, TimerState::SingleShot);
//         assert_eq!(shim.get_state(T0).unwrap(), TimerState::SingleShot);
//     }

//     #[test]
//      fn get_repeated() {
//         let mut shim = TimersShim::new();
//         let res = shim.set_state(T0, TimerState::Repeated);
//         assert_eq!(shim.get_state(T0).unwrap(), TimerState::Repeated);
//     }

//     #[test]
//      fn get_set_period_singleshot() {
//         let mut shim = TimersShim::new();
//         let res = shim.set_state(T0, TimerState::SingleShot);
//         shim.set_period(T0, 200);
//         assert_eq!(shim.get_period(T0).unwrap(), 200);
//     }

//     #[test]
//      fn get_set_period_repeated() {
//         let mut shim = TimersShim::new();
//         let res = shim.set_state(T0, TimerState::Repeated);
//         shim.set_period(T0, 200);
//         assert_eq!(shim.get_period(T0).unwrap(), 200);
//     }

// }
