use core::num::NonZeroU8;
use lc3_traits::peripherals::pwm::{
    Pwm, PwmPin, PwmPinArr, PwmSetDutyError, PwmSetPeriodError, PwmState,
};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::u8::MAX;
use timer;
use chrono;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;


pub struct PwmShim {
    states: PwmPinArr<PwmState>,
    duty_cycle: PwmPinArr<u8>,
    guards1: PwmPinArr<Option<timer::Guard>>,
    guards2: PwmPinArr<Option<timer::Guard>>,
    bit_states: Arc<Mutex<PwmPinArr<bool>>>,
    timers    : [timer::Timer; 2]
}

impl Default for PwmShim {
    fn default() -> Self {
        //let pins = [Arc::new(Mutex::new(false)), Arc::new(Mutex::new(false))];

        Self {
            states: PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]),
            duty_cycle: PwmPinArr([0; PwmPin::NUM_PINS]), // start with duty_cycle low
            guards1: PwmPinArr([None, None]),
            guards2: PwmPinArr([None, None]),
            bit_states: Arc::new(Mutex::new(PwmPinArr([false, false]))),
            timers: [timer::Timer::new(), timer::Timer::new()]
        }
    }
}

impl PwmShim {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_pin_state(&self, pin: PwmPin) -> PwmState {
        self.states[pin].into()
    }

    pub fn set_duty_cycle_helper(&mut self, pin: PwmPin, period: NonZeroU8) {


        let timer = timer::Timer::new();


        let (tx, rx) = channel();

        let guard1 = {

            let pin_cl = self.bit_states.clone();
            self.timers[0].schedule_repeating(chrono::Duration::milliseconds(MAX as i64), move || {


                (*pin_cl.lock().unwrap())[pin]=true;

            })
        };


        let _guard2s = timer.schedule_with_delay(chrono::Duration::milliseconds((self.duty_cycle[pin]) as i64), move || {
            let _ignored = tx.send(());

        });

        rx.recv().unwrap();


        let guard2 = {

            let pin_cl = self.bit_states.clone();

            self.timers[1].schedule_repeating(chrono::Duration::milliseconds(MAX as i64), move || {

                (*pin_cl.lock().unwrap())[pin]=false;

            })
        };
        self.guards1[pin] = Some(guard1);
        self.guards2[pin] = Some(guard2);

        }

    fn start_timer(&mut self, pin: PwmPin, state: PwmState) {
            use PwmState::*;
            match state {
                Enabled(time) => {
                    match self.guards1[pin] {
                        Some(_) => {
                            self.stop_timer(pin);
                            if self.duty_cycle[pin] != 0 {
                                self.set_duty_cycle_helper(pin, time);

                            }
                        }
                        None => {
                            if self.duty_cycle[pin] != 0 {
                                self.set_duty_cycle_helper(pin, time);
                            }
                        }
                    }
                }
                Disabled => {}
            }
     }

    fn stop_timer(&mut self, pin: PwmPin) {
            match self.guards1[pin] {
                Some(_) => {
                    let g = self.guards1[pin].take().unwrap();
                    drop(g);
                    self.guards1[pin] = None;
                    let g2 = self.guards2[pin].take().unwrap();
                    drop(g2);
                    self.guards2[pin] = None;
                }
                None => { }
            }
    }
}
impl Pwm for PwmShim {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError> {
        use PwmState::*;
        self.states[pin] = match state {
            Enabled(time) => {

                self.start_timer(pin, state);
                Enabled(time)
            }
            Disabled => {
                self.stop_timer(pin);
                Disabled
            }
        };
        Ok(())
    }

    fn get_state(&self, pin: PwmPin) -> PwmState {
        self.states[pin]
    }

    fn get_pin(&self, pin: PwmPin) -> bool {
        return (*self.bit_states.lock().unwrap())[pin];
    }

    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError> {
        self.duty_cycle[pin] = duty;

        self.start_timer(pin, self.states[pin]);

        Ok(())
    }

    fn get_duty_cycle(&self, pin: PwmPin) -> u8 {
        self.duty_cycle[pin]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::pwm::{self, Pwm, PwmPin::*, PwmState};

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
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX).unwrap()));
        assert_eq!(res, Ok(()));
        let val = shim.get_state(P0);
        assert_eq!(val, pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
    }

    #[test]
    fn get_duty() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX).unwrap()));
        assert_eq!(res, Ok(()));
        let res2 = shim.set_duty_cycle(P0, 100);
        assert_eq!(res2, Ok(()));
        assert_eq!(shim.get_duty_cycle(P0), 100);
        shim.set_state(P0, pwm::PwmState::Disabled);
    }

    #[test]
    fn get_pin_initial() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX).unwrap()));

        let b = shim.get_pin(P0);
        assert_eq!(b, false);
    }

    #[test]
    fn get_pin_on() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX).unwrap()));

        let res = shim.set_duty_cycle(P0, MAX); // should always be on
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
        fn test_duty_cycle() {

            let mut shim = PwmShim::new();

            let res = shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
            assert_eq!(res, Ok(()));

            shim.set_duty_cycle(P0, MAX/2);
            thread::sleep(Duration::from_millis(MAX as u64)); // run twice then disable
            shim.set_state(P0, pwm::PwmState::Disabled);

        }




}
