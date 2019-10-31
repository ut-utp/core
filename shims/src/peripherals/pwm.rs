use lc3_traits::peripherals::pwm::{NUM_PWM_PINS, PwmState, PwmPinArr, Pwm, PwmPin};
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
            State::Enabled(_) => Enabled(NonZeroU8::new(1).unwrap()), // when decalring enabled, let 1 denote the default val
            State::Disabled => Disabled,
        }
    }
}

pub struct PwmShim {
    states: PwmPinArr<State>,
    period: PwmPinArr<u8>,
    duty_cycle: PwmPinArr<u8>, 
    is_disabled: bool, // in order to kill the signal 
}

impl Default for PwmShim {
    fn default() -> Self {
        Self {
            states: [State::Disabled; NUM_PWM_PINS as usize],
            period: [0; NUM_PWM_PINS as usize],      //cycles: [0; NUM_PWM_PINS as usize], <- remove because duty cycle doesn't care about particular pins 
            duty_cycle: [0; NUM_PWM_PINS as usize],  // start with duty_cycle low
            is_disabled: true, 
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
                self.is_disabled = false;
                State::Enabled(false)
            }, 
            Disabled => {
                self.period[usize::from(pin)] = 0;
                self.is_disabled = false;
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


// consider this code to be the work of a child or someone you really don't trust
    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8){
       //use State::*;
        self.duty_cycle[usize::from(pin)] = duty;
        
        let (tx, rx) = mpsc::channel::<State>();

        let on_period = (self.duty_cycle[usize::from(pin)]/MAX) * self.period[usize::from(pin)]; // get the on period 
        let off_period = self.period[usize::from(pin)] - on_period; // get the off period

        
        let state = Mutex::new(Cell::new(self.states[usize::from(pin)]));

        if self.is_disabled {
            let _disable = tx.send(State::Disabled);
        }

        let _handle = thread::spawn(move || {
            // probably not correct, but now the compiler errors are gone
            loop{
                
                match rx.recv() {
                    Ok(State::Disabled) => {break;}
                    Ok(State::Enabled(_)) => {}
                    Err(_) => {}
                }
                // shared state can only be accessed when the lock is held
                let mut state_data = state.lock().unwrap(); 

                *state_data.get_mut() = State::Enabled(true); // self.states[usize::from(pin)] = Enabled(true);
                thread::sleep(Duration::from_millis(on_period as u64));

               *state_data.get_mut() = State::Enabled(false); // self.states[usize::from(pin)] = Enabled(false);
                thread::sleep(Duration::from_millis(off_period as u64));
            
                
            }
        });

       

        
    }
   

}

