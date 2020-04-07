
use lc3_traits::peripherals::timers::{
    Timers, TimerArr, TimerId, TimerMode, TimerState, Period
};
use lc3_traits::control::Snapshot;

use timer;

use std::sync::Arc;
use std::time::Instant;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct TimersShim<'a> {
    states: TimerArr<TimerState>,
    modes: TimerArr<TimerMode>,

    external_flags: Option<&'a TimerArr<AtomicBool>>,
    internal_flags: Arc<TimerArr<AtomicBool>>,

    guards: TimerArr<Option<timer::Guard>>,
    timers: TimerArr<timer::Timer>,

    start_times: TimerArr<Option<Instant>>,
}

macro_rules! arr { ($v:expr) => { TimerArr([$v, $v]) }; }

impl Default for TimersShim<'_> {
    fn default() -> Self {
       Self {
            states: arr!(TimerState::Disabled),
            modes: arr!(TimerMode::SingleShot),

            external_flags: None,
            internal_flags: Arc::new(arr!(AtomicBool::new(false))),

            guards: arr!(None),
            timers: arr!(timer::Timer::new()),

            start_times: arr!(None),
        }
    }
}

impl TimersShim<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    fn start_timer(&mut self, timer: TimerId, period: Period) {
        use TimerMode::*;

        let duration = chrono::Duration::milliseconds(period.get() as i64);
        let flags = self.internal_flags.clone();

        // Register the current time as the start time for this timer:
        self.start_times[timer] = Some(Instant::now());

        let guard = match self.get_mode(timer) {
            Repeated => {
                self.timers[timer].schedule_repeating(duration, move || {
                    flags[timer].store(true, Ordering::SeqCst)
                })
            },
            SingleShot => {
                self.timers[timer].schedule_with_delay(duration, move || {
                    flags[timer].store(true, Ordering::SeqCst)
                })
            },
        };

        self.guards[timer] = Some(guard);
    }

    fn stop_timer(&mut self, timer: TimerId) {
        drop(self.guards[timer].take())
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

        // Stop any existing `timer::Timer`s for this timer:
        self.stop_timer(timer);

        if let WithPeriod(period) = state {
            self.start_timer(timer, period);
        }

        self.states[timer] = state;
    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        self.states[timer]
    }

    fn register_interrupt_flags(&mut self, flags: &'a TimerArr<AtomicBool>) {
        self.external_flags = match self.external_flags {
            None => Some(flags),
            Some(_) => {
                // warn!("re-registering interrupt flags!");
                Some(flags)
            }
        }
    }

    // Whenever we're 'polled' about the state of a timer, we'll update the
    // external flags.
    fn interrupt_occurred(&self, timer: TimerId) -> bool {
        use Ordering::SeqCst;

        let occurred = self.external_flags.unwrap()[timer].load(SeqCst);
        self.internal_flags[timer].store(occurred, SeqCst);

        self.interrupts_enabled(timer) && occurred
    }

    // Here, we'll clear both the internal and external flags.
    fn reset_interrupt_flag(&mut self, timer: TimerId) {
        use Ordering::SeqCst;

        self.external_flags.unwrap()[timer].store(false, SeqCst);
        self.internal_flags[timer].store(false, SeqCst);
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

    static FLAGS_GSPS: TimerArr<AtomicBool> = TimerArr([AtomicBool::new(false), AtomicBool::new(false)]);
    #[test]
    fn get_set_period_singleshot() {
        let mut shim = TimersShim::new();
        shim.register_interrupt_flags(&FLAGS_GSPS);
        let res = shim.set_mode(T0, TimerMode::SingleShot);
        let period = NonZeroU16::new(200).unwrap();
        shim.set_state(T0, TimerState::WithPeriod(period));
        assert_eq!(shim.get_state(T0), TimerState::WithPeriod(period));
    }

    static FLAGS_GSPR: TimerArr<AtomicBool> = TimerArr([AtomicBool::new(false), AtomicBool::new(false)]);
    #[test]
    fn get_set_period_repeated() {
        let mut shim = TimersShim::new();
        shim.register_interrupt_flags(&FLAGS_GSPR);
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

       let sleep = Duration::from_millis(205);
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
