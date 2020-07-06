not_wasm! {
use core::num::NonZeroU8;
use lc3_traits::peripherals::pwm::{
    Pwm, PwmPin, PwmPinArr, PwmState, PwmDutyCycle,
};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use timer;
use chrono;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::atomic::Ordering::SeqCst;
use std::thread::sleep;

const MAX_PERIOD: u8 = u8::max_value();
const MAX_DUTY_CYCLE: PwmDutyCycle = PwmDutyCycle::max_value();

pub struct PwmShim {
    states: PwmPinArr<PwmState>,
    duty_cycle: PwmPinArr<PwmDutyCycle>,
    rising_edge_guards: PwmPinArr<Option<timer::Guard>>,
    falling_edge_guards: PwmPinArr<Option<timer::Guard>>,
    bit_states: Arc<PwmPinArr<AtomicBool>>,
    timers: PwmPinArr<timer::Timer>,
}

impl Default for PwmShim {
    fn default() -> Self {
        //let pins = [Arc::new(Mutex::new(false)), Arc::new(Mutex::new(false))];

        Self {
            states: PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]),
            duty_cycle: PwmPinArr([0; PwmPin::NUM_PINS]), // start with duty_cycle low
            rising_edge_guards: PwmPinArr([None, None]),
            falling_edge_guards: PwmPinArr([None, None]),
            bit_states: Arc::new(PwmPinArr([AtomicBool::new(false), AtomicBool::new(false)])),
            timers: PwmPinArr([timer::Timer::new(), timer::Timer::new()])
        }
    }
}

impl PwmShim {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: remove?
    pub fn get_pin_state(&self, pin: PwmPin) -> PwmState {
        self.states[pin].into()
    }

    fn start_wave(&mut self, pin: PwmPin, period: NonZeroU8) {
        self.stop_wave(pin);

        let period = period.get();
        let duration = chrono::Duration::milliseconds(period as i64);

        self.bit_states[pin].store(true, SeqCst); // start with rising edge

        // Schedule future rising edges
        let pin_clone = self.bit_states.clone();
        let rising_edge_guard = self.timers[pin].schedule_repeating(duration, move | | {
            pin_clone[pin].store(true, SeqCst);
        });

        // Wait to schedule falling edges
        let high_time = (period as u64) * (self.duty_cycle[pin] as u64) / (MAX_DUTY_CYCLE as u64);
        sleep(Duration::from_millis(high_time));

        self.bit_states[pin].store(true, SeqCst); // make a falling edge

        // Schedule future falling edges
        let pin_clone = self.bit_states.clone();
        let falling_edge_guard = self.timers[pin].schedule_repeating(duration, move | | {
            pin_clone[pin].store(false, SeqCst);
        });

        self.rising_edge_guards[pin] = Some(rising_edge_guard);
        self.falling_edge_guards[pin] = Some(falling_edge_guard);
    }

    fn stop_wave(&mut self, pin: PwmPin) {
        let reg = self.rising_edge_guards[pin].take();
        drop(reg);
        let feg = self.falling_edge_guards[pin].take();
        drop(feg);
    }

    fn get_pin(&self, pin: PwmPin) -> bool {
        return self.bit_states[pin].load(SeqCst);
    }

}

impl Pwm for PwmShim {
    fn set_state(&mut self, pin: PwmPin, state: PwmState)  {
        use PwmState::*;
        match state {
            Enabled(period) => {
                self.start_wave(pin, period);
            }
            Disabled => {
                self.stop_wave(pin);
            }
        };
        self.states[pin] = state;
    }

    fn get_state(&self, pin: PwmPin) -> PwmState {
        self.states[pin]
    }

    fn set_duty_cycle(&mut self, pin: PwmPin, duty: PwmDutyCycle) {
        self.duty_cycle[pin] = duty;
        if let PwmState::Enabled(period) = self.states[pin] {
            self.start_wave(pin, period);
        }
    }

    fn get_duty_cycle(&self, pin: PwmPin) -> PwmDutyCycle {
        self.duty_cycle[pin]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::pwm::{self, Pwm, PwmPin::*, PwmState};

    use lc3_test_infrastructure::{
        assert_eq, assert_is_about, run_periodically_for_a_time
    };
    #[test]
    fn get_disabled() {
        let mut shim = PwmShim::new();
        assert_eq!(shim.get_state(P0), PwmState::Disabled);

        let res = shim.set_state(P1, PwmState::Disabled);
        assert_eq!(shim.get_state(P0), PwmState::Disabled);
    }

    #[test]
    fn get_enabled() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX_PERIOD).unwrap()));
        let val = shim.get_state(P0);
        assert_eq!(val, pwm::PwmState::Enabled((NonZeroU8::new(MAX_PERIOD)).unwrap()));
    }

    #[test]
    fn get_duty() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX_PERIOD).unwrap()));
        let res2 = shim.set_duty_cycle(P0, 100);
        assert_eq!(shim.get_duty_cycle(P0), 100);
        shim.set_state(P0, pwm::PwmState::Disabled);
    }

    #[test]
    fn get_pin_initial() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX_PERIOD).unwrap()));

        let b = shim.get_pin(P0);
        assert_eq!(b, true);
    }

    #[test]
    fn get_pin_on() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX_PERIOD).unwrap()));

        let res = shim.set_duty_cycle(P0, MAX_DUTY_CYCLE); // should always be on
        thread::sleep(Duration::from_millis(10));
        let b = shim.get_pin(P0);
        assert_eq!(b, true);
    }

    #[test]
    fn start_pwm() {
        let mut shim = PwmShim::new();
        let res0 = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(255).unwrap()));
        let res1 = shim.set_duty_cycle(P0, 100); // this starts pwm

        let b = shim.get_pin(P0);
        thread::sleep(Duration::from_millis(100));
        let b2 = shim.get_pin(P0);

        assert_eq!(b, b2);
    }

    #[test]
    fn P0_toggle_once_check() {
        let mut shim = PwmShim::new();

        let res = shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX_PERIOD)).unwrap()));

        shim.set_duty_cycle(P0, MAX_DUTY_CYCLE / 2);
        let pin_state = shim.get_pin(P0);
        thread::sleep(Duration::from_millis(1 as u64));
        let mut toggle_flag = 0;
        for i in 0..4 { // let it  run for 2 complete cycles
            if shim.get_pin(P0) != pin_state {
                toggle_flag = 1;
            }
            thread::sleep(Duration::from_millis((MAX_DUTY_CYCLE/2) as u64));
        }
        assert_eq!(toggle_flag,  1);
        shim.set_state(P0, pwm::PwmState::Disabled);
    }


    #[test]
    fn P1_toggle_once_check() {
        let mut shim = PwmShim::new();

        let res = shim.set_state(P1, pwm::PwmState::Enabled((NonZeroU8::new(MAX_PERIOD)).unwrap()));

        shim.set_duty_cycle(P1, MAX_DUTY_CYCLE / 2);
        let pin_state = shim.get_pin(P1);

        thread::sleep(Duration::from_millis(1 as u64));
        let mut toggle_flag = 0;
        for i in 0..4 { // let it run for 2 complete cycles
            if shim.get_pin(P1) != pin_state {
                toggle_flag = 1;
            }
            thread::sleep(Duration::from_millis((MAX_DUTY_CYCLE/2) as u64));
        }
        assert_eq!(toggle_flag,  1);

        shim.set_state(P1, pwm::PwmState::Disabled);
    }


    #[test]
    fn P0_duty_cycle() {

        let mut shim = PwmShim::new();

        shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX_DUTY_CYCLE)).unwrap()));

        let num_cycles = 5;
        let duty_cycle_ratio: u8 = 5;



        let mut p0_count = 0;

        let oncycle = (MAX_DUTY_CYCLE/duty_cycle_ratio) as u64;
        let offcycle = (MAX_DUTY_CYCLE - MAX_DUTY_CYCLE/duty_cycle_ratio) as u64;

        shim.set_duty_cycle(P0, MAX_DUTY_CYCLE/duty_cycle_ratio);
        thread::sleep(Duration::from_millis(1 as u64));  // give wiggle room

        let mut actual_cycles = Arc::new(Mutex::new(0));

        let timer = timer::Timer::new();
        let duration = chrono::Duration::milliseconds(MAX_DUTY_CYCLE as i64);



        let guard = {
            let actual_cycles = actual_cycles.clone();
            timer.schedule_repeating(duration, move | | {
                *actual_cycles.lock().unwrap() += 1;
            })
        };
        while p0_count < num_cycles {
            let state_p0 = shim.get_pin(P0); // on
            if state_p0 == true {
                thread::sleep(Duration::from_millis(oncycle));
            } else {
                thread::sleep(Duration::from_millis(offcycle));
            }
            let new_state_p0 = shim.get_pin(P0);

            if new_state_p0 == true {
                thread::sleep(Duration::from_millis(oncycle));
            } else {
                thread::sleep(Duration::from_millis(offcycle));
            }

            if state_p0 != new_state_p0 { // we are looking for num_cycles # of toggles
                p0_count += 1;
            }



        };
        shim.set_state(P0, pwm::PwmState::Disabled);
        drop(guard);

        let p0_check = p0_count >= num_cycles;

       assert_eq!(p0_check, true);
       assert_eq!(*actual_cycles.lock().unwrap(), num_cycles);


    }

    #[test]
    fn P1_duty_cycle() {

        let mut shim = PwmShim::new();

        shim.set_state(P1, pwm::PwmState::Enabled((NonZeroU8::new(MAX_DUTY_CYCLE)).unwrap()));

        let num_cycles = 5;
        let duty_cycle_ratio: u8 = 5;
        let mut p1_count = 0;

        let oncycle = (MAX_DUTY_CYCLE/duty_cycle_ratio) as u64;
        let offcycle = (MAX_DUTY_CYCLE - MAX_DUTY_CYCLE/duty_cycle_ratio) as u64;

        shim.set_duty_cycle(P1, MAX_DUTY_CYCLE/duty_cycle_ratio);
        thread::sleep(Duration::from_millis(1 as u64));  // give wiggle room

        let mut actual_cycles = Arc::new(Mutex::new(0));

        let timer = timer::Timer::new();
        let duration = chrono::Duration::milliseconds(MAX_DUTY_CYCLE as i64);



        let guard = {
            let actual_cycles = actual_cycles.clone();
            timer.schedule_repeating(duration, move | | {
                *actual_cycles.lock().unwrap() += 1;
            })
        };
        while p1_count < num_cycles {
            let state_p1 = shim.get_pin(P1); // on
            if state_p1 == true {
                thread::sleep(Duration::from_millis(oncycle));
            } else {
                thread::sleep(Duration::from_millis(offcycle));
            }
            let new_state_p1 = shim.get_pin(P1);

            if new_state_p1 == true {
                thread::sleep(Duration::from_millis(oncycle));
            } else {
                thread::sleep(Duration::from_millis(offcycle));
            }

            if state_p1 != new_state_p1 { // we are looking for num_cycles # of toggles
                p1_count += 1;
            }



        };
        shim.set_state(P1, pwm::PwmState::Disabled);
        drop(guard);

        let p1_check = p1_count >= num_cycles;

       assert_eq!(p1_check, true);
       assert_eq!(*actual_cycles.lock().unwrap(), num_cycles);


    }
        // #[test]
        // fn test_multi_duty_cycle() {

        //     let mut shim = PwmShim::new();

        //     shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX_DUTY_CYCLE)).unwrap()));
        //     shim.set_state(P1, pwm::PwmState::Enabled((NonZeroU8::new(MAX_DUTY_CYCLE)).unwrap()));

        //     let num_cycles = 5;
        //     let duty_cycle_ratio: u8 = 5;



        //     let mut p0_count = 0;
        //     let mut p1_count = 0;
        //     let oncycle = (MAX_DUTY_CYCLE/duty_cycle_ratio) as u64;
        //     let offcycle = (MAX_DUTY_CYCLE - MAX_DUTY_CYCLE/duty_cycle_ratio) as u64;
        //     shim.set_duty_cycle(P0, MAX_DUTY_CYCLE/duty_cycle_ratio);
        //     shim.set_duty_cycle(P1, MAX_DUTY_CYCLE/duty_cycle_ratio);



        //    thread::sleep(Duration::from_millis(1 as u64));  // give wiggle room

        //     let mut actual_cycles = Arc::new(Mutex::new(0));

        //     let timer = timer::Timer::new();
        //     let duration = chrono::Duration::milliseconds(MAX_DUTY_CYCLE as i64);



        //     let guard = {
        //         let actual_cycles = actual_cycles.clone();
        //         timer.schedule_repeating(duration, move | | {
        //             *actual_cycles.lock().unwrap() += 1;
        //         })
        //     };



        //     while p0_count < num_cycles || p1_count < num_cycles {
        //         let state_p0 = shim.get_pin(P0); // on
        //         let state_p1 = shim.get_pin(P1); // on

        //         if state_p0 == true || state_p1 == true{ // not good, these aren't happening at similar enough times
        //             thread::sleep(Duration::from_millis(oncycle));
        //         } else {
        //             thread::sleep(Duration::from_millis(offcycle));
        //         }

        //         let new_state_p0 = shim.get_pin(P0);
        //         let new_state_p1 = shim.get_pin(P1);

        //         if new_state_p0 == true || new_state_p1 == true {
        //             thread::sleep(Duration::from_millis(oncycle));
        //         } else {
        //             thread::sleep(Duration::from_millis(offcycle));
        //         }


        //         if state_p0 != new_state_p0 { // we are looking for num_cycles # of toggles
        //             p0_count += 1;
        //         }

        //         if state_p1 != new_state_p1 {
        //             p1_count += 1;
        //         }


        //     };
        //     shim.set_state(P0, pwm::PwmState::Disabled);
        //     shim.set_state(P1, pwm::PwmState::Disabled);
        //     drop(guard);

        //     let p0_check = p0_count >= num_cycles;
        //     let p1_check = p1_count >= num_cycles;
        //    assert_eq!(p0_check, true);
        //    assert_eq!(*actual_cycles.lock().unwrap(), num_cycles);
        // }


}
}

wasm! {
    pub use lc3_traits::peripherals::stubs::PwmStub as PwmShim;
}
