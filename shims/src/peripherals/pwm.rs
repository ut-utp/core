use lc3_traits::peripherals::pwm::Pwm;

use std::thread;
//use core::ops::{Index, IndexMut};
use std::sync::{Arc, RwLock};

pub enum PwmPin { P0, P1 }


pub struct PwmShim {
    states: PwmPinArr<PwmState>,
    duty_cycle: u16,
}

impl Default for PwmShim {
    fn default() -> Self {
        Self {
            states: [PwmState::Disabled; NUM_PWM_PINS as usize],
            //cycles: [0; NUM_PWM_PINS as usize], <- remove because duty cycle doesn't care about particular pins 
            duty_cycle: 0,  // start with duty_cycle low
        }
    }
}

impl PwmShim {
    pub fn new() -> Self{
        Self::default()
    }
    pub fn get_pin_state(&self, pin: PwmPin) -> PwmState {
        self.states[pin].into()
    }
}


impl Pwm for PwmShim {
     fn set_state(&mut self, pin: u8, state: PwmState) -> Result<(), ()>{
        self.states[pin] = state;
        ok(())
     }

     fn get_state(&self, pin: u8) -> Option<PwmState>{
        //  if pin < 2 { 
        //      return Some(self.states[pin]);
        //  } else {
        //      return None;
        //      }
       if(pin < 2) {
           if let PwmState::Disabled = self.get_pin_state(pin) {
               return None;
           }
           return some(self.get_pin_state(pin));
       } else {
           return None;
       }
        

     }

    fn set_duty_cycle(&self, duty: u16){ 
        // not mutable - should set duty cycle just start here? 
        // doesn't give ability to set duty cycles for each pin?
        
        // won't work bc not mutable
        self.duty_cycle = duty; // duty cycle is a percentage of u16 - u16_max_value() is 100% duty



    }
    // fn set_high(&mut self, pin: u8){
    //         // set what to  high..?

    // }

    fn start(&mut self, pin: u8){
        // fn set_high(){}
        // timer.register_interrupt(free pin, set_high);
        // timer.set_period(free pin, duty);
     //   use crate::peripherals::clock;

        

// whatever state your implementing this on, have a bank of 4 booleans on 
        
        
        //timer.register_interrupt(0, self.set_high(0); // pretend like this sets bit high
        //timer.set_period(0, self.duty_cycle*clock::get_nanosecond();

        
        // when interrupt occurs 

        // timer.register_interrupt(0, self.set_low(0));
        // timer.set_period(0, clock::get_nanoseconds() - self.duty_cycle*clock::get_nanosecond());


        // fn set_low(){}
        // timer.register_interrupt(free pin, set_low);
        // timer.set_period(free pin, clock::get_nanoseconds() - duty);

        


       // set interrupt for system period - % on = % off
       // start interrupt
       // interrupt occurs
       // set high for duty cycle % of period
        // set low for % off  

        // better way to do this? Would this even work? Would we not want to invoke timers - needed for other purposes? 
        // basically, we can just set a timer to the off period (total period - total period * (% duty cycle))
        // and handler = set bit high and change timer period to the on period (% duty cycle of total period)
        // then set handler = set bit low and change timer back to off period
    
    } //Start the periodic timer interrupt
    fn disable(&mut self, pin: u8){
        // disable the period timer interrupt 
        if(pin < 2){
            self.states[pin] = Disabled;
        }
        

    }


}

