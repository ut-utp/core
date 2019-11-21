use core::num::NonZeroU8;
use lc3_traits::peripherals::pwm::{
    Pwm, PwmPin, PwmPinArr, PwmSetDutyError, PwmSetPeriodError, PwmState,
};
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::u8::MAX;
extern crate timer;
extern crate time;
use core::sync::atomic::{AtomicBool, Ordering};
//use core::ops::{Index, IndexMut};

static PWM_SHIM_PINS: PwmPinArr<AtomicBool> = PwmPinArr([AtomicBool::new(false), AtomicBool::new(false)]);

pub struct PwmShim {
    states: PwmPinArr<PwmState>,
    duty_cycle: PwmPinArr<u8>,
    guards: PwmPinArr<Option<timer::Guard>>,
}

impl Default for PwmShim {
    fn default() -> Self {
        //let pins = [Arc::new(Mutex::new(false)), Arc::new(Mutex::new(false))];
       
        Self {
            states: PwmPinArr([PwmState::Disabled; PwmPin::NUM_PINS]),
            duty_cycle: PwmPinArr([0; PwmPin::NUM_PINS]), // start with duty_cycle low
            guards: PwmPinArr([None, None]),
            
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
    
    pub fn set_duty_cycle_helper(&mut self, pin: PwmPin, period: NonZeroU8){
        use PwmState::*;
       
        let duty = self.duty_cycle[pin]; 
        let time_on = period.get()*(duty/MAX); 
        let time_off = period.get() - time_on;
        let timer_init = timer::Timer::new();

        let guard = {
           
                // start ASAP
            timer_init.schedule_with_delay(time::Duration::milliseconds(0), move || {
               
                loop {
                    // the incredibly short intervals of time where the value is stored is making tests fail
                    // so need to have these gross if-statements
                   if time_off != period.get() {
                        PWM_SHIM_PINS[pin].store(true, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(time_on as u64)); 
                   }
                    if time_on != period.get() {
                        PWM_SHIM_PINS[pin].store(false, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(time_off as u64));
                    }
                    
                    
                }
            })


        };
        // thread::sleep(Duration::from_nanos(1)); 
        self.guards[pin] = Some(guard);
    }

    fn start_timer(&mut self, pin: PwmPin, state: PwmState){
        use PwmState::*;
        match state {
            Enabled(time) => {
                match self.guards[pin] {
                    Some(_) => { // if there is a guard in action, drop it and go
                        self.stop_timer(pin);
                        if self.duty_cycle[pin] != 0 {
                            self.set_duty_cycle_helper(pin, time);
                        } // should I throw an error if they don't set the duty cycle?
                        
                    },
                    None => { // if there is no guard in action
                        if self.duty_cycle[pin] != 0 {
                            self.set_duty_cycle_helper(pin, time);
                        }
                    }
                }
            },
            Disabled => {} 
    // if it is disabled, then you shouldn't be trying to start it - should I throw an error?
        }
    }

    fn stop_timer(&mut self, pin: PwmPin){
        match self.guards[pin]{ 
            Some(_) => { // if there is a guard, drop it 
                let g = self.guards[pin].take().unwrap();
                drop(g);
                self.guards[pin] = None;
            },
            None => {} // if there isn't, don't do anything 
        }
        
    }
}
impl Pwm for PwmShim {
    fn set_state(&mut self, pin: PwmPin, state: PwmState) -> Result<(), PwmSetPeriodError> {
       use PwmState::*;
        self.states[pin] = match state{
            Enabled(time) => {
                // start PWM - this method will drop any PWM that's already in action 
                self.start_timer(pin, state);
                Enabled(time)
            },
            Disabled => {
                // stop PWM
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
        return PWM_SHIM_PINS[pin].load(Ordering::SeqCst);
    }

    fn set_duty_cycle(&mut self, pin: PwmPin, duty: u8) -> Result<(), PwmSetDutyError> {
        
        self.duty_cycle[pin] = duty;
        // the reason you actually call this here
        // is to change duty cycles without having to change state
        // it won't start on it's own before setting state to enabled 
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

//         let res2 = shim.set_duty_cycle(P0, MAX); // should always be on
//         thread::sleep(Duration::from_millis(10));
//         let b2 = shim.get_pin(P0);
//         assert_eq!(b2, true);
    }
    
     #[test]
    fn get_pin_on() {
        let mut shim = PwmShim::new();
        let res = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(MAX).unwrap()));

        //let b = shim.get_pin(P0);
         //assert_eq!(b, false); 

        let res = shim.set_duty_cycle(P0, MAX); // should always be on
        thread::sleep(Duration::from_millis(10));
        let b = shim.get_pin(P0);
        assert_eq!(b, true);


    }


    // #[test]
    // fn start_pwm() {
    //     let mut shim = PwmShim::new();
    //     let res0 = shim.set_state(P0, pwm::PwmState::Enabled(NonZeroU8::new(255).unwrap()));
    //     let res1 = shim.set_duty_cycle(P0, 100); // this starts pwm 
    //     // duty cycle = 100/255, off cycle = 155/255
        
    //     let b = shim.get_pin(P0);
    //     thread::sleep(Duration::from_millis(100));
    //     let b2 = shim.get_pin(P0); 

    //     assert_eq!(b, b2);
    // }


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
}
