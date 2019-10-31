use lc3_traits::peripherals::pwm::{NUM_PWM_PINS, PwmState, PwmPinArr, Pwm, PwmPin, PwmMiscError};
use std::u8::MAX;
use std::thread;
use std::time::Duration;
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::mpsc;
use core::num::NonZeroU8;
//use core::ops::{Index, IndexMut};
use std::sync::{Arc, RwLock};

#[derive(Copy, Clone, Debug)]
pub enum State { 
    Enabled(bool),
    Disabled,
}

impl From<State> for PwmState {
    fn from(state: State) -> PwmState {
        use PwmState::*;
        match state {
            State::Enabled(_) => Enabled(NonZeroU8::new(MAX).unwrap()), // when decalring enabled, let 1 denote the default val
            State::Disabled => Disabled,
        }
    }
}

pub struct PwmShim {
    states: PwmPinArr<State>,
    period: PwmPinArr<u8>,
    duty_cycle: PwmPinArr<u8>, 
    thread_open: PwmPinArr<bool>, // in order to kill the signal 
}

impl Default for PwmShim {
    fn default() -> Self {
        Self {
            states: [State::Disabled; NUM_PWM_PINS as usize],
            period: [0; NUM_PWM_PINS as usize],      //cycles: [0; NUM_PWM_PINS as usize], <- remove because duty cycle doesn't care about particular pins 
            duty_cycle: [0; NUM_PWM_PINS as usize],  // start with duty_cycle low
            thread_open: [false; NUM_PWM_PINS as usize], 
        }
    }
}

impl PwmShim {
    pub fn new() -> Self{
        Self::default()
    }
    pub fn get_pin_state(&self, pin: PwmPin) -> PwmState {
        self.states[usize::from(pin)].into()
    }

}




impl Pwm for PwmShim {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), ()>{
        use PwmState::*;
        self.states[usize::from(pin)] = match state {
            Enabled(time) => {
                self.period[usize::from(pin)] = time.get();
                //self.is_disabled = false;
                State::Enabled(false)
            }, 
            Disabled => {
                self.period[usize::from(pin)] = 0;
                //self.is_disabled = false;
                self.thread_open[usize::from(pin)] = false;
                self.set_duty_cycle(pin, 0); // arbitrary - will disable duty cycle anyway
                State::Disabled
                },
        };

        Ok(())
    }

    fn get_state(&self, pin: PwmPin) -> Option<PwmState> {
        if let PwmState::Disabled = self.get_pin_state(pin) {
            None
        } else {
            Some(self.get_pin_state(pin))
        }
    }



    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8){
       //use State::*;
        self.duty_cycle[usize::from(pin)] = duty;
        
        let (tx, rx) = mpsc::channel::<State>();
        
        // if the duty cycle for this pin is not running, create it 
        if self.thread_open[usize::from(pin)] == false {
        // get on period and off period by percentage duty cycle 
        // try to hold on to significant digits through division... will inaccuracy become a problem??
        let on_period = (((self.duty_cycle[usize::from(pin)] as f64)/(MAX as f64)) as u8) * self.period[usize::from(pin)]; // get the on period 
        let off_period = self.period[usize::from(pin)] - on_period; // get the off period

         if self.period[usize::from(pin)] == 0 { 
            let _disable = tx.send(State::Disabled);
        }
        let state = Mutex::new(Cell::new(self.states[usize::from(pin)]));
        // period can only be 0 if the signal should be disabled
       
        let _handle = thread::spawn(move || {
            // probably not correct, but now the compiler errors are gone
            loop{
                
                match rx.recv() {
                    Ok(State::Disabled) => {break;}
                    Ok(State::Enabled(_)) => {}
                    Err(_) => {
                        // print some message?
                        break;

                    }
                }
                // shared state can only be accessed when the lock is held
                let mut state_data = state.lock().unwrap(); 
        
                *state_data.get_mut() = State::Enabled(true); // self.states[usize::from(pin)] = Enabled(true);
                thread::sleep(Duration::from_millis(on_period as u64));
            
               *state_data.get_mut() = State::Enabled(false); // self.states[usize::from(pin)] = Enabled(false);
                thread::sleep(Duration::from_millis(off_period as u64));
            
                
            }
        });
        self.thread_open[usize::from(pin)]= true;
        } else {
            // otherwise disable the current thread and then 
             let _disable = tx.send(State::Disabled); // consumes before I call duty cycle again? 
             self.thread_open[usize::from(pin)] = false;
             self.set_duty_cycle(pin, duty); // open the thread with new duty cycle
            return; 
        
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::pwm::{self, Pwm, PwmPin::*,};

    #[test]
    fn get_disabled() {
        let shim = PwmShim::new();
        assert_eq!(shim.get_state(P0), None);
    }
    
    #[test]
    fn get_enabled() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
        assert_eq!(res, Ok(()));
        let val = shim.get_state(P0);
        // as long as it says enabled, it should pass this test... 
        // we don't actually reset the value of the period within enabled...
        assert_eq!(val.unwrap(), pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
    }


    #[test]
    fn test_duty_cycle() {

        let mut shim = PwmShim::new();
      
        let res = shim.set_state(P0, pwm::PwmState::Enabled((NonZeroU8::new(MAX)).unwrap()));
        assert_eq!(res, Ok(()));

        shim.set_duty_cycle(P0, MAX/2);
        thread::sleep(Duration::from_millis(MAX as u64)); // run twice then disable 
        shim.set_state(P0, pwm::PwmState::Disabled);
        //let val = shim.get_state(P0);

       // assert_eq!(val.unwrap(), pwm::PwmState::Enabled((NonZeroU8::new(MAX/2)).unwrap()));

        // how tf do I test this guy...?
    }
}
