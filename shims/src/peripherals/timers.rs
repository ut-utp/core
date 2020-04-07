
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

        let occurred = self.internal_flags[timer].load(SeqCst);
        self.external_flags.unwrap()[timer].store(occurred, SeqCst);

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
    use lc3_traits::peripherals::timers::{
        TimerId::*, TimerMode::*, TimerState::*
    };

    use lc3_test_infrastructure::{
        assert_eq, assert_is_about, run_periodically_for_a_time
    };

    use std::time::Duration;
    use std::num::NonZeroU16;

    #[test]
    fn get_disabled() {
        let shim = TimersShim::new();
        assert_eq!(shim.get_state(T0), Disabled);
        assert_eq!(shim.get_state(T1), Disabled);
    }

    #[test]
    fn default_mode_is_singleshot() {
        assert_eq!(TimersShim::new().get_mode(T0), SingleShot);
    }

    #[test]
    fn get_singleshot() {
        let mut shim = TimersShim::new();
        shim.set_mode(T0, SingleShot);
        assert_eq!(shim.get_mode(T0), SingleShot);
    }

    #[test]
    fn get_repeated() {
        let mut shim = TimersShim::new();
        shim.set_mode(T0, Repeated);
        assert_eq!(shim.get_mode(T0), Repeated);

        // T1 should still be in single shot mode.
        assert_eq!(shim.get_mode(T1), SingleShot);
    }

    macro_rules! shim {
        () => {{
            let mut _shim = TimersShim::new();
            _shim.register_interrupt_flags(shim!(flags));
            _shim
        }};
        (flags) => {{
            static _FLAGS: TimerArr<AtomicBool> = arr!(AtomicBool::new(false));
            &_FLAGS
        }};
    }

    macro_rules! p { ($expr:expr) => {WithPeriod(NonZeroU16::new($expr).unwrap())}; }

    #[test]
    fn get_set_period_singleshot() {
        let mut shim = shim!();

        shim.set_mode(T0, SingleShot);
        shim.set_state(T0, p!(200));

        shim.set_mode(T1, SingleShot);
        shim.set_state(T1, p!(1024));

        assert_eq!(shim.get_state(T0), p!(200));
        assert_eq!(shim.get_state(T1), p!(1024));
    }

    #[test]
    fn setting_mode_disables_timer() {
        let mut shim = shim!();

        shim.set_state(T0, p!(20_000));
        assert_eq!(shim.get_mode(T0), SingleShot);
        assert_eq!(shim.get_state(T0), p!(20_000));

        // Even though this is the mode we're already in, this should disable
        // the timer.
        shim.set_mode(T0, SingleShot);
        assert_eq!(shim.get_state(T0), Disabled);
    }

    #[test]
    fn get_set_period_repeated() {
        let mut shim = shim!();

        shim.set_mode(T1, Repeated);
        shim.set_state(T1, p!(65_535));

        assert_eq!(shim.get_mode(T1), Repeated);
        assert_eq!(shim.get_state(T1), p!(65_535));

        assert_eq!(shim.get_mode(T0), SingleShot);
        assert_eq!(shim.get_state(T0), Disabled);
    }

    #[test]
    fn get_singleshot_interrupt_occurred() {
        let mut shim = shim!();

        shim.set_mode(T0, SingleShot);
        shim.set_state(T0, p!(200));

        let record = run_periodically_for_a_time(
            Duration::from_millis(20),   // Every 20 milliseconds..
            Duration::from_millis(240),  // ..for the next 240 milliseconds..
            move |_| shim.interrupt_occurred(T0), // ..check if T0 fired.
        );

        for (time, fired) in &record {
            let expected = time.as_millis() > 200;

            assert_eq!(
                *fired,
                expected,
                "Expected T0 (SingleShot, 200ms) to {} fired at {:?}. \
                Full record: {:?}.",
                if expected { "have" } else { "have not" },
                time,
                record,
            );
        }
   }


    #[test]
    fn concurrent_singleshot_and_repeated() {
        let mut shim = shim!();

        shim.set_mode(T0, SingleShot);
        shim.set_state(T0, p!(200));

        shim.set_mode(T1, Repeated);
        shim.set_state(T1, p!(50));

        let record = run_periodically_for_a_time(
            Duration::from_millis(24),
            Duration::from_millis(240),
            move |_| {
                let res = (shim.interrupt_occurred(T0), shim.interrupt_occurred(T1));

                if res.0 { shim.reset_interrupt_flag(T0) }
                if res.1 { shim.reset_interrupt_flag(T1) }

                res
            }
        );

        // Check T0's record:
        let mut fired_on_last_step = false;
        let mut num_times_fired = 0;
        for (t, (f0, _)) in &record {
            if fired_on_last_step {
                assert_eq!(*f0, false, "T0's `interrupt_occurred` failed to reset at {:?}.", t);
            }

            if *f0 { num_times_fired += 1; }

            fired_on_last_step = *f0;
        }

        assert_eq!(num_times_fired, 1);
        let fired_at = record.iter().map(|(t, (f, _))| (t, f)).filter(|(_, f)| **f).next().unwrap();
        assert_is_about(fired_at.0.as_millis() as u16, 200, 10);

        // Check T1's record:
        let mut fired_on_last_step = false;
        let mut num_times_fired = 0;
        for (t, (_, f1)) in &record {
            if fired_on_last_step {
                assert_eq!(*f1, false, "T0's `interrupt_occurred` failed to reset at {:?}.", t);
            }

            if *f1 { num_times_fired += 1; }

            fired_on_last_step = *f1;
        }

        assert_eq!(num_times_fired, 4);
        record.iter()
            .map(|(t, (_, f))| (t, f))
            .filter(|(_, f)| **f)
            .map(|(t, _)| t.as_millis() as u16)
            .enumerate()
            .for_each(|(idx, t)| assert_is_about(t, idx as u16 * 50, 2));
    }

   //  #[test]
   //  fn get_repeated_interrupt_occured() {
   //      let mut shim = shim!();

   //      shim.set_mode(T0, TimerMode::Repeated);
   //      let period = NonZeroU16::new(200).unwrap();

   //      shim.set_state(T0, TimerState::WithPeriod(period));
   //      let mut bool_arr = Vec::<bool>::new();
   //      let sleep = Duration::from_millis(205);

   //      let mut count = 0;
   //      for i in 1..=5 {
   //          thread::sleep(sleep);
   //          if shim.interrupt_occurred(T0) {
   //              count += 1;
   //              shim.reset_interrupt_flag(T0);
   //          }
   //      }
   //      assert_eq!(count, 5);
   // }
}
