not_wasm!{
use lc3_traits::peripherals::timers::{
    Timers, TimerArr, TimerId, TimerMode, TimerState, Period, TIMERS
};
use lc3_traits::control::Snapshot;

use timer;

use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct TimersShim<'tint> {
    states: Arc<TimerArr<Mutex<TimerState>>>,
    modes: TimerArr<TimerMode>,

    external_flags: Option<&'tint TimerArr<AtomicBool>>,
    internal_flags: Arc<TimerArr<AtomicBool>>,

    guards: TimerArr<Option<timer::Guard>>,
    timers: TimerArr<timer::Timer>,

    start_times: TimerArr<Option<Instant>>,
}

macro_rules! arr { ($v:expr) => { TimerArr([$v, $v]) }; }

impl Default for TimersShim<'_> {
    fn default() -> Self {
       Self {
            states: Arc::new(arr!(Mutex::new(TimerState::Disabled))),
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
                let states = self.states.clone();

                self.timers[timer].schedule_with_delay(duration, move || {
                    flags[timer].store(true, Ordering::SeqCst);

                    let mut state = states[timer].lock().unwrap();

                    *state = TimerState::Disabled;
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

        let mut guard = self.states[timer].lock().unwrap();
        *guard = state;
    }

    fn get_state(&self, timer: TimerId) -> TimerState {
        *self.states[timer].lock().unwrap()
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

        /*self.interrupts_enabled(timer) && */occurred
    }

    // Here, we'll clear both the internal and external flags.
    fn reset_interrupt_flag(&mut self, timer: TimerId) {
        use Ordering::SeqCst;

        self.external_flags.unwrap()[timer].store(false, SeqCst);
        self.internal_flags[timer].store(false, SeqCst);
    }
}

#[derive(Debug)]
pub struct TimersSnapshot {
    states: TimerArr<TimerState>,
    modes: TimerArr<TimerMode>,

    flags: TimerArr<bool>,
    start_times: TimerArr<Option<Instant>>,
    snapshot_time: Instant,
}

impl<'a> Snapshot for TimersShim<'a> {
    type Snap = TimersSnapshot;
    type Err = core::convert::Infallible;

    fn record(&self) -> Result<Self::Snap, Self::Err> {
        Ok(TimersSnapshot {
            states: TimerArr([
                *self.states[TimerId::T0].lock().unwrap(),
                *self.states[TimerId::T1].lock().unwrap(),
            ]),
            modes: self.modes.clone(),

            flags: TimerArr([
                self.internal_flags[TimerId::T0].load(Ordering::SeqCst),
                self.internal_flags[TimerId::T1].load(Ordering::SeqCst),
            ]),
            start_times: self.start_times.clone(),
            snapshot_time: Instant::now(),
        })
    }

    fn restore(&mut self, snap: Self::Snap) -> Result<(), Self::Err> {
        // Stop all running timers:
        TIMERS.iter().for_each(|t| self.stop_timer(*t));

        // States and modes we can restore without much fuss. Flags too.
        TIMERS.iter().for_each(|t| {
            let mut state = self.states[*t].lock().unwrap();

            *state = snap.states[*t];
        });
        self.modes = snap.modes;

        self.start_times = snap.start_times;

        for t in TIMERS.iter() {
            self.internal_flags[*t].store(snap.flags[*t], Ordering::SeqCst);
            self.external_flags.unwrap()[*t].store(snap.flags[*t], Ordering::SeqCst);
        }

        // The problem is dealing with timers that were already running at the
        // time the snapshot was taken.
        for t in TIMERS.iter() {
            use TimerState::*;
            use TimerMode::*;

            fn remaining_time(start: &Option<Instant>, snap_time: Instant, p: Period) -> (Duration, Duration) {
                let start_time = start
                        .expect("running timers should have a start time");

                let elapsed = snap_time.duration_since(start_time);
                let remaining = Duration::from_millis(p.get().into()) - elapsed;

                (elapsed, remaining)
            }

            let state = *self.states[*t].lock().unwrap();
            match (state, self.modes[*t]) {
                // For SingleShot timers, this is simple enough:
                (WithPeriod(p), SingleShot) => {
                    // We just need to schedule the timer again _once_ for the
                    // time it would have fired at. All we need to do is
                    // calculate how much time was left on it's clock:
                    // let start_time = self.start_times[*t]
                    //     .expect("running timers should have a start time");

                    // let elapsed = snap.snapshot_time.duration_since(start_time);
                    // let remaining = Duration::from_millis(p.get().into()) - elapsed;

                    let (elapsed, remaining) = remaining_time(&self.start_times[*t], snap.snapshot_time, p);

                    // And schedule a timer for that time:
                    self.start_timer(*t, Period::new(remaining.as_millis().max(1) as u16).unwrap());

                    // Update the start time to reflect how much of the period
                    // has already elapsed (in case a snapshot is taken again
                    // before this timer fires (or is dropped)).
                    *self.start_times[*t].as_mut().unwrap() -= elapsed;
                },

                // Repeated timers are a little tricker.
                (WithPeriod(p), Repeated) => {
                    // We want to run the timer next after the remaining time
                    // (like with the singleshot timer) but then every period
                    // *starting after the remaining amount of time*.
                    //
                    // Luckily, `timer` has our back; `timer::Timer::schedule`
                    // does exactly this.
                    let (elapsed, remaining) = remaining_time(&self.start_times[*t], snap.snapshot_time, p);

                    let remaining = chrono::Duration::from_std(remaining).unwrap();
                    let period = chrono::Duration::milliseconds(p.get() as i64);
                    let flags = self.internal_flags.clone();

                    let guard = self.timers[*t].schedule(chrono::Utc::now() + remaining, Some(period), move || {
                        flags[*t].store(true, Ordering::SeqCst)
                    });

                    self.guards[*t] = Some(guard);

                    // As with SingleShot, update the start time to reflect the
                    // progress made.
                    self.start_times[*t] = Some(Instant::now() - elapsed);
                },

                (Disabled, _) => {},
            }
        }

        Ok(())
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

    use std::thread::sleep;

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

    macro_rules! p { ($expr:expr) => {WithPeriod(Period::new($expr).unwrap())}; }

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

        // Give it some wiggle room:
        sleep(Duration::from_millis(1));

        let mut fired = false;

        let record = run_periodically_for_a_time(
            Duration::from_millis(20),   // Every 20 milliseconds..
            Duration::from_millis(500),  // ..for the next 500 milliseconds..
            move |_| {
                let res = shim.interrupt_occurred(T0);
                if res { shim.reset_interrupt_flag(T0); fired = true; }

                if fired {
                    assert_eq!(shim.get_state(T0), Disabled,
                        "Once a SingleShot timer fires, it should disable itself.");

                    assert_eq!(shim.interrupts_enabled(T0), false,
                        "Once a SingleShot timer fires and is checked, interrupts should be disabled.");
                }

                res
            }, // ..check if T0 fired and so on.
        );

        let mut already_fired = false;
        for (time, fired) in &record {
            let expected = (time.as_millis() >= 200) && !already_fired;

            assert_eq!(
                *fired,
                expected,
                "Expected T0 (SingleShot, 200ms) to {} fired at {:?}. \
                Full record: {:?}.",
                if expected { "have" } else { "have not" },
                time,
                record,
            );

            if *fired { already_fired = true; }
        }
   }

    #[test]
    fn concurrent_singleshot_and_repeated() {
        let mut shim = shim!();

        shim.set_mode(T0, SingleShot);
        shim.set_state(T0, p!(200));

        shim.set_mode(T1, Repeated);
        shim.set_state(T1, p!(50));

        // Give it some wiggle room:
        sleep(Duration::from_millis(1));

        let record = run_periodically_for_a_time(
            Duration::from_millis(10),
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
        assert_is_about(fired_at.0.as_millis() as u16, 200, 2);

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
            .for_each(|(idx, t)| assert_is_about(t, (idx + 1) as u16 * 50, 2));
    }

    #[test]
    fn get_repeated_interrupt_occurred() {
        let mut shim = shim!();

        shim.set_mode(T0, Repeated);
        shim.set_state(T0, p!(200));

        let sleep_time = Duration::from_millis(200);

        // Wiggle room.
        sleep(Duration::from_millis(2));

        let mut count = 0;
        for _ in 1..=5 {
            sleep(sleep_time);
            if shim.interrupt_occurred(T0) {
                count += 1;
                shim.reset_interrupt_flag(T0);
            }
        }

        assert_eq!(count, 5);
    }
}
}

wasm! {
    #[derive(Debug, Default)]
    pub struct TimersShim<'t>(PhantomData<&'t ()>);

    use lc3_traits::peripherals::timers::{Timers, TimerId, TimerArr, TimerMode, TimerState};
    use core::sync::atomic::AtomicBool;
    use core::marker::PhantomData;
    impl<'a> Timers<'a> for TimersShim<'a> {
        fn set_mode(&mut self, _timer: TimerId, _mode: TimerMode) { }
        fn get_mode(&self, _timer: TimerId) -> TimerMode { TimerMode::SingleShot }

        fn set_state(&mut self, _timer: TimerId, _state: TimerState) { }
        fn get_state(&self, _timer: TimerId) -> TimerState { TimerState::Disabled }

        fn register_interrupt_flags(&mut self, _flags: &'a TimerArr<AtomicBool>) {}
        fn interrupt_occurred(&self, _timer: TimerId) -> bool { false }
        fn reset_interrupt_flag(&mut self, _timer: TimerId) { }
    }
}
