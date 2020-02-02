use lc3_traits::peripherals::timers::{
    TimerArr, TimerId, TimerMiscError, TimerState, TimerStateMismatch, Timers,
};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
//use std::time::Duration;
use core::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use time;
use timer;

// The term “Single Shot” signifies a single pulse output of some duration.
#[derive(Clone)] // TODO: Debug
pub struct TimersShim<'a> {
    states: TimerArr<TimerState>,
    times: TimerArr<Word>,
    flags: Option<&'a TimerArr<AtomicBool>>,
    guards: TimerArr<Option<timer::Guard>>,
}

impl Default for TimersShim<'_> {
    fn default() -> Self {
        Self {
            states: TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]),
            times: TimerArr([0u16; TimerId::NUM_TIMERS]), // unlike gpio, interrupts occur on time - not on bit change
            flags: None,
            guards: TimerArr([None, None]),
        }
    }
}

impl TimersShim<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Timers<'a> for TimersShim<'a> {
    fn set_state(&mut self, timer: TimerId, state: TimerState) -> Result<(), TimerMiscError> {
        use TimerState::*;
        self.states[timer] = match state {
            Repeated => {
                match self.guards[timer] {
                    Some(_) => {
                        let g = self.guards[timer].take().unwrap();
                        drop(g);
                        // drop(x);
                        state
                    }
                    None => state,
                }
            }
            SingleShot => {
                match self.guards[timer] {
                    Some(_) => {
                        let g = self.guards[timer].take().unwrap();
                        drop(g);
                        //drop(x);
                        state
                    }
                    None => state,
                }
            }
            Disabled => state,
        };

        Ok(())
    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        self.states[timer]
    }

    fn set_period(&mut self, timer: TimerId, milliseconds: Word) -> Result<(), TimerMiscError> {
        //  use TimerState::*;
        //  self.times[timer] = milliseconds;
        //  let timer_init = timer::Timer::new();

        // match self.guards[timer] {
        //      Some(_) => {
        //          let g = self.guards[timer].take().unwrap();
        //          drop(g);

        //      },
        //      None => {}
        //  }

        //  match self.states[timer] {
        //      Repeated => {
        //          match self.flags[timer] {
        //              Some(b) => {
        //                  let guard = {
        //                      timer_init.schedule_repeating(time::Duration::milliseconds(milliseconds as i64), move || {
        //                      //self.flags[timer].unwrap().store(true, Ordering::SeqCst);
        //                      b.store(true, Ordering::SeqCst);
        //                      })

        //                  };

        //                  self.guards[timer] = Some(guard);
        //              },
        //              None => {
        //                  unreachable!();
        //              }

        //          }
        //      },
        //      SingleShot => {
        //          match self.flags[timer] {
        //              Some(b) => {
        //                  let guard = {
        //                          timer_init.schedule_with_delay(time::Duration::milliseconds(milliseconds as i64), move || {
        //                      //self.flags[timer].unwrap().store(true, Ordering::SeqCst);
        //                          b.store(true, Ordering::SeqCst);
        //                      })
        //                   };

        //                   self.guards[timer] = Some(guard);
        //              }
        //              None => {
        //                  unreachable!();
        //              }
        //          }
        //      },
        //      Disabled => {
        //          unreachable!();
        //      }

        //  }

        //  Ok(())

        unimplemented!()
    }

    fn get_period(&self, timer: TimerId) -> Word {
        self.times[timer]
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {
        // TODO: decide what we want to do for repeated settings of this (see gpio's shim)

        self.flags = match self.flags {
            None => Some(flags),
            Some(_) => unreachable!(),
        }
    }

    fn interrupt_occurred(&self, timer: TimerId) -> bool {
        match self.flags {
            Some(flags) => {
                let occurred = flags[timer].load(Ordering::SeqCst);
                self.interrupts_enabled(timer) && occurred
            }
            None => unreachable!(),
        }
    }

    fn reset_interrupt_flag(&mut self, timer: TimerId) {
        match self.flags {
            Some(flag) => flag[timer].store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
    }

    // TODO: review whether we want Interrupt state or interrupts_enabled bool state
    fn interrupts_enabled(&self, timer: TimerId) -> bool {
        match self.get_state(timer) {
            SingleShot => true,
            Repeating => true,
            Disabled => false,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use lc3_traits::peripherals::timers::{Timer::*, Timers};

//     use pretty_assertions::assert_eq;

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
