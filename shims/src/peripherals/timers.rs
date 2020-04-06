use lc3_traits::peripherals::timers::{
    Timers, TimerArr, TimerId, TimerMode, TimerState,
};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
//use std::time::Duration;

use std::sync::Arc;
use std::sync::Mutex;
use core::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use core::num::NonZeroU16;
use time;
use timer;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;
// The term “Single Shot” signifies a single pulse output of some duration.

pub struct TimersShim<'a> {
    states: TimerArr<TimerState>,
    modes: TimerArr<TimerMode>,
    times: TimerArr<NonZeroU16>,
    flags: Option<&'a TimerArr<AtomicBool>>,
    flags1: Arc<Mutex<TimerArr<bool>>>,
    guards: TimerArr<Option<timer::Guard>>,
    timer1: TimerArr<timer::Timer>,
}

impl Default for TimersShim<'_> {
    fn default() -> Self {
       Self {
            states: TimerArr([TimerState::Disabled; TimerId::NUM_TIMERS]),
            modes: TimerArr([TimerMode::SingleShot; TimerId::NUM_TIMERS]),
            times: TimerArr([NonZeroU16::new(1).unwrap(); TimerId::NUM_TIMERS]),
            flags: None,
            flags1: Arc::new(Mutex::new(TimerArr([false, false]))),
            guards: TimerArr([None, None]),
            timer1: TimerArr([timer::Timer::new(), timer::Timer::new()]),


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


                        let guard1 = {
                            let flag_cl = self.flags1.clone();

                            self.timer1[timer].schedule_repeating(chrono::Duration::milliseconds(period.get() as i64), move || {

                                (*flag_cl.lock().unwrap())[timer]=true;
                            })
                        };

                        self.guards[timer] = Some(guard1);


                    },
                    SingleShot => {
                        //let timer1 = timer::Timer::new();
                        let (tx, rx) = channel();
                        let guard1 = self.timer1[timer].schedule_with_delay(chrono::Duration::milliseconds(period.get() as i64), move || {
                            let _ignored = tx.send(());
                        });

                        rx.recv().unwrap();

                        self.guards[timer] = Some(guard1);
                        match self.flags {
                            Some(flag) => flag[timer].store(true, Ordering::SeqCst),
                            None => {},
                        };
                    }
                }

            },
            Disabled => {}

        }




    }

    fn stop_timer(&mut self, timer: TimerId) {
        if let Some(guard) = self.guards[timer].take() {
            drop(guard);
        }
    }

}

impl<'a> Timers<'a> for TimersShim<'a> {
    fn set_mode(&mut self, timer: TimerId, mode: TimerMode) {
        self.set_state(timer, TimerState::Disabled);
        self.modes[timer] = mode;
    }

    fn get_mode(&self, timer: TimerId) -> TimerMode {
        self.modes[timer]
    }

    fn set_state(&mut self, timer: TimerId, state: TimerState) {
        use TimerState::*;

        match state {
            WithPeriod(period) => {
                self.times[timer] = period;
                self.start_timer(timer, self.modes[timer], state);
            },
            Disabled => {
                self.stop_timer(timer);
            }
        };

        self.states[timer] = state;
    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        self.states[timer]
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {

        self.flags = match self.flags {
            None => Some(flags),
            Some(_) => {
                // warn!("re-registering interrupt flags!");
                Some(flags)
            }
        }
    }

    fn interrupt_occurred(&self, timer: TimerId) -> bool {
        use TimerMode::*;
        let occurred = match self.modes[timer] {
            Repeated => {
                (*self.flags1.lock().unwrap())[timer]
            },
            SingleShot => {
                let flags = self.flags.unwrap();
                flags[timer].load(Ordering::SeqCst)
            },
        };
        self.interrupts_enabled(timer) && occurred
    }

    fn reset_interrupt_flag(&mut self, timer: TimerId) {
        match self.flags {
            Some(flag) => flag[timer].store(false, Ordering::SeqCst),
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::timers::{TimerId::*, Timers};

    use lc3_test_infrastructure::assert_eq;

    #[test]
    fn get_disabled() {
        let shim = TimersShim::new();
        assert_eq!(shim.get_state(T0), TimerState::Disabled);
    }

    #[test]
     fn get_singleshot() {
        let mut shim = TimersShim::new();
        let res = shim.set_mode(T0, TimerMode::SingleShot);
        assert_eq!(shim.get_mode(T0), TimerMode::SingleShot);
    }

    #[test]
     fn get_repeated() {
        let mut shim = TimersShim::new();
        let res = shim.set_mode(T0, TimerMode::Repeated);
        assert_eq!(shim.get_mode(T0), TimerMode::Repeated);
    }

    #[test]
     fn get_set_period_singleshot() {
        let mut shim = TimersShim::new();
        let res = shim.set_mode(T0, TimerMode::SingleShot);
        let period = NonZeroU16::new(200).unwrap();
        shim.set_state(T0, TimerState::WithPeriod(period));
        assert_eq!(shim.get_state(T0), TimerState::WithPeriod(period));
    }

    #[test]
     fn get_set_period_repeated() {
        let mut shim = TimersShim::new();
        let res = shim.set_mode(T0, TimerMode::Repeated);
        let period = NonZeroU16::new(200).unwrap();
        shim.set_state(T0, TimerState::WithPeriod(period));
        assert_eq!(shim.get_state(T0), TimerState::WithPeriod(period));
    }


    static FLAGS: TimerArr<AtomicBool> = TimerArr([AtomicBool::new(false), AtomicBool::new(false)]);
    #[test]
    fn get_singleshot_interrupt_occured() {
       let mut shim = TimersShim::new();
       shim.register_interrupt_flags(&FLAGS);
       shim.set_mode(T0, TimerMode::SingleShot);
       let period = NonZeroU16::new(200).unwrap();

       shim.set_state(T0, TimerState::WithPeriod(period));

       let sleep = Duration::from_millis(200);
       thread::sleep(sleep);
       assert_eq!(shim.interrupt_occurred(T0), true);
   }


    static FLAGS2: TimerArr<AtomicBool> = TimerArr([AtomicBool::new(false), AtomicBool::new(false)]);
    #[test]
    fn get_repeated_interrupt_occured() {
        let mut shim = TimersShim::new();
        shim.register_interrupt_flags(&FLAGS2);
        shim.set_mode(T0, TimerMode::Repeated);
        let period = NonZeroU16::new(200).unwrap();

        shim.set_state(T0, TimerState::WithPeriod(period));
        let mut bool_arr = Vec::<bool>::new();
        let sleep = Duration::from_millis(205);

        let mut count = 0;
        for i in 1..=5 {
            thread::sleep(sleep);
            if shim.interrupt_occurred(T0) {
                count += 1;
                shim.reset_interrupt_flag(T0);
            }
        }
        assert_eq!(count, 5);
   }



}
