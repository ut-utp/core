use lc3_traits::peripherals::timers::{
    TimerArr, TimerHandler, TimerId, TimerMiscError, TimerState, TimerStateMismatch, Timers,
};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
use std::cell::Cell;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// The term “Single Shot” signifies a single pulse output of some duration.
pub struct TimersShim {
    states: TimerArr<TimerState>,
    times: TimerArr<Word>,
    handlers: TimerArr<TimerHandler<'static>>, // handlers for timers
}

const NO_OP: TimerHandler<'static> = &|_| {};

impl Default for TimersShim {
    fn default() -> Self {
        Self {
            states: TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]),
            times: TimerArr([0u16; TimerId::NUM_TIMERS]), // unlike gpio, interrupts occur on time - not on bit change
            handlers: TimerArr([NO_OP; TimerId::NUM_TIMERS]),
        }
    }
}

impl TimersShim {
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

impl Timers<'static> for TimersShim {
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

    fn register_interrupt(
        &mut self,
        timer: TimerId,
        func: TimerHandler<'static>,
    ) -> Result<(), TimerMiscError> {
        // self.handlers[timer] = func;
        // Ok(())
        unimplemented!()
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
