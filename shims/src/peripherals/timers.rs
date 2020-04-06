use lc3_traits::peripherals::timers::{
    Timers, TimerArr, TimerId, TimerMode, TimerState,
};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
//use std::time::Duration;
use core::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use core::num::NonZeroU16;
use time;
use timer;
use std::sync::mpsc::channel;
// The term “Single Shot” signifies a single pulse output of some duration.

pub struct TimersShim<'a> {
    states: TimerArr<TimerState>,
    modes: TimerArr<TimerMode>,
    times: TimerArr<NonZeroU16>,
    flags: Option<&'a TimerArr<AtomicBool>>,
    guards: TimerArr<Option<timer::Guard>>,
    timer1: timer::Timer
}

impl Default for TimersShim<'_> {
    fn default() -> Self {
       Self {
            states: TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]),
            modes: TimerArr([TimerMode::SingleShot; TimerId::NUM_TIMERS]),
            times: TimerArr([NonZeroU16::new(1).unwrap(); TimerId::NUM_TIMERS]),
            flags: None,
            guards: TimerArr([None, None]),
            timer1: timer::Timer::new()

        }
    }
}

impl TimersShim<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_timer(&mut self, timer: TimerId, mode: TimerMode, state: TimerState) {
        use TimerMode::*;
        use TimerState::*;

         match state {
            WithPeriod(period) => {
                match mode {

                    Repeated => {
                        let timer1 = timer::Timer::new();
                        let max = 65536;
                        let (tx, rx) = channel();
                        let guard1 = {
                            self.timer1.schedule_repeating(chrono::Duration::milliseconds(max as i64), move || {})
                        };
                        let _guard2s = timer1.schedule_with_delay(chrono::Duration::milliseconds(i64::from(self.times[timer].get())), move || {
                            let _ignored = tx.send(());

                        });

                        rx.recv().unwrap();
                        match self.flags {
                            Some(flag) => flag[timer].store(true, Ordering::SeqCst),
                            None => unreachable!(),
                        }
                        self.guards[timer] = Some(guard1);
                    },
                    SingleShot => {
                        let timer1 = timer::Timer::new();
                        let max = 65536;
                        let (tx, rx) = channel();
                        let guard = timer1.schedule_with_delay(chrono::Duration::milliseconds(i64::from(self.times[timer].get())), move || {
                            let _ignored = tx.send(());
                        });

                        rx.recv().unwrap();
                        match self.flags {
                            Some(flag) => flag[timer].store(true, Ordering::SeqCst),
                            None => unreachable!(),
                        }
                        self.guards[timer] = Some(guard);
                    }
                }

            },
            Disabled => {}

        }




    }


}

impl<'a> Timers<'a> for TimersShim<'a> {
    fn set_mode(&mut self, timer: TimerId, mode: TimerMode) {
        use TimerMode::*;
        self.modes[timer] = match mode {
            Repeated => {
                match self.guards[timer] {
                    Some(_) => {
                        let g = self.guards[timer].take().unwrap();
                        drop(g);
                        self.states[timer] = TimerState::Disabled;
                        mode
                    }
                    None => mode,
                }
            }
            SingleShot => {
                match self.guards[timer] {
                    Some(_) => {
                        let g = self.guards[timer].take().unwrap();
                        drop(g);
                        self.states[timer] = TimerState::Disabled;
                        mode
                    }
                    None => mode,
                }
            }
            Disabled => mode,
        };


    }

    fn get_mode(&self, timer: TimerId) -> TimerMode {
        self.modes[timer]
    }


    fn set_state(&mut self, timer: TimerId, state: TimerState) {
    use TimerState::*;

    self.states[timer] = match state {
        WithPeriod(period) => {
            if period == NonZeroU16::new(0).unwrap() {
                let g = self.guards[timer].take().unwrap();
                drop(g);
                state
            } else {
                self.times[timer] = period;
                self.start_timer(timer, self.modes[timer], self.states[timer]);
                state
            }
        },
        Disabled => {
            state
        }
    };

    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        self.states[timer]
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {

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
