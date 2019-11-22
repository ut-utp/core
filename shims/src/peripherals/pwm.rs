use core::num::NonZeroU8;
use lc3_traits::peripherals::pwm::{
    Pwm, PwmPin, PwmPinArr, PwmSetDutyError, PwmSetPeriodError, PwmState,
};
use std::cell::Cell;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::u8::MAX;
//use core::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct PwmShim {
    states: PwmPinArr<PwmState>,
    pins: PwmPinArr<Mutex<bool>>,
    duty_cycle: PwmPinArr<u8>,
    thread_open: PwmPinArr<bool>, // in order to kill the signal
}

impl Default for PwmShim {
    fn default() -> Self {
        let pins = [Mutex::new(false), Mutex::new(false)];

        Self {
            states: PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]),
            pins: PwmPinArr(pins),                        // pins start low
            duty_cycle: PwmPinArr([0; PwmPin::NUM_PINS]), // start with duty_cycle low
            thread_open: PwmPinArr([false; PwmPin::NUM_PINS]),
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
}

impl Pwm for PwmShim {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError> {
        // use PwmState::*;
        // self.states[pin] = match state {
        //     Enabled(time) => {
        //         self.period[pin] = time.get();
        //         //self.is_disabled = false;
        //         State::Enabled(false)
        //     }
        //     Disabled => {
        //         self.period[pin] = 0;
        //         //self.is_disabled = false;
        //         self.thread_open[pin] = false;
        //         self.set_duty_cycle(pin, 0); // arbitrary - will disable duty cycle anyway
        //         State::Disabled
        //     }
        // };

        // Ok(())
        unimplemented!()
    }

    fn get_state(&self, pin: PwmPin) -> PwmState {
        self.states[pin]
    }

    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError> {
        //use State::*;
        // self.duty_cycle[pin] = duty;

        // let (tx, rx) = mpsc::channel::<State>();

        // // if the duty cycle for this pin is not running, create it
        // if self.thread_open[pin] == false {
        //     // get on period and off period by percentage duty cycle
        //     // try to hold on to significant digits through division... will inaccuracy become a problem??
        //     let on_period = (((self.duty_cycle[pin] as f64) / (MAX as f64)) as u8)
        //         * self.period[pin]; // get the on period
        //     let off_period = self.period[pin] - on_period; // get the off period

        //     if self.period[pin] == 0 {
        //         let _disable = tx.send(State::Disabled);
        //     }
        //     let state = Mutex::new(Cell::new(self.states[pin]));

        //     let _handle = thread::spawn(move || {
        //         loop {
        //             match rx.recv() {
        //                 Ok(State::Disabled) => {
        //                     break;
        //                 }
        //                 Ok(State::Enabled(_)) => {}
        //                 Err(_) => {
        //                     // print some message?
        //                     break;
        //                 }
        //             }

        //             let mut state_data = state.lock().unwrap();

        //             *state_data.get_mut() = State::Enabled(true); // self.states[usize::from(pin)] = Enabled(true);
        //             thread::sleep(Duration::from_millis(on_period as u64));

        //             *state_data.get_mut() = State::Enabled(false); // self.states[usize::from(pin)] = Enabled(false);
        //             thread::sleep(Duration::from_millis(off_period as u64));
        //         }
        //     });
        //     self.thread_open[pin] = true;
        // } else {
        //     // otherwise disable the current thread and then
        //     let _disable = tx.send(State::Disabled); // consumes before I call duty cycle again?
        //     self.thread_open[pin] = false;
        //     self.set_duty_cycle(pin, duty); // open the thread with new duty cycle
        //     return;
        // }
        unimplemented!()
    }

    fn get_duty_cycle(&self, pin: PwmPin) -> u8 {
        unimplemented!()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use lc3_traits::peripherals::pwm::{self, Pwm, PwmPin::*, PwmState};

//     #[test]
//     fn get_disabled() {
//         let shim = PwmShim::new();
//         assert_eq!(shim.get_state(P0), PwmState::Disabled);
//     }

//     #[test]
//     fn get_enabled() {
//         let mut shim = PwmShim::new();
//         let res = shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
//         assert_eq!(res, Ok(()));
//         let val = shim.get_state(P0);
//          assert_eq!(val.unwrap(), pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
//     }

//     #[test]
//     fn test_duty_cycle() {

//         let mut shim = PwmShim::new();

//         let res = shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
//         assert_eq!(res, Ok(()));

//         shim.set_duty_cycle(P0, MAX/2);
//         thread::sleep(Duration::from_millis(MAX as u64)); // run twice then disable
//         shim.set_state(P0, pwm::PwmState::Disabled);

//     }
// }
