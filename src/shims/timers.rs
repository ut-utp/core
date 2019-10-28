// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use crate::peripherals::timers::Timers; // define timer errors
use crate::Word;
use core::ops::{Index, IndexMut};
use std::sync::{Arc, RwLock};
extern crate cortex_m::peripheral::syst;

pub enum TimerEnum { T0, T1 }

#[derive(Copy, Clone, Debug)]
pub enum State {
    Repeated(bool), // need a way to remember which state timer is in
    SingleShot(bool),
   // ask why no interrupt state: Interrupt(bool),
    Disabled,
}



impl From<State> for TimerState {
    fn from(state: State) -> TimerState {
        use TimerState::*;

        match state {
            State::Repeated(_) => Repeated,
            State::SingleShot(_) => SingleShot, 
            State::Disabled => Disabled,
        }
    }
}

// impl From<TimerEnum> for usize {
//     fn from(timer: TimerEnum) -> usize {
//         use TimerEnum::*;
//         match timer {
//             T0 => 0,
//             T1 => 1,
//         }
//     }
// }

// The term “Single Shot” signifies a single pulse output of some duration. 
pub struct TimersShim {
     states: TimerArr<State>,
     times: TimerArr<Word>, 
     handlers: TimerArr<&'a dyn Fn(GpioPin)>, // handlers for timers
}

// always access timers with u8, so no necessary indexes



impl Default for TimerShim<'_> {
    fn default() -> Self {
        Self {
            states: [State::Disabled; NUM_TIMERS as usize],
            times: [None; NUM_TIMERS as usize], // unlike gpio, interrupts occur on time - not on bit change
            handlers: [NO_OP; NUM_TIMERS as usize],
        }
    }
}

impl TimersShim<'a>{
     pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Timers<'a> for TimersShim<'a> {
    fn set_state(&mut self, num: u8, state: TimerState) -> Result<(), ()>{
        use TimerState::*;
    
        self.states[num] = match state {
            Repeated => State::Repeated(true), // true? 
            SingleShot => State::SingleShot(true), // true?
            Disabled => State::Disabled,
        };
        Ok(())
    }
    fn get_state(&mut self, num: u8) -> Option<TimerState>{ 
        // why is self mutable? Just getting...
        Some(self.states[num].into())

    }
  fn set_period(&mut self, num: u8, milliseconds: Word){ 
      // thread based

  }






    fn set_period(&mut self, num: u8, milliseconds: Word){ 
        // automatically starts when period is set?
        // what about single shot timers
        // AKA should I implement a start timer method

        // without a start method, I would just start the timer off here
        // so this method is dependent on if that's true

       // for now, assume we have start(&self, num: u8)
       
       // set the period to milliseconds if !disabled
//        use crate::peripherals::clock::*;

    



    // how to separate timers
     
        

        //self.times[num] = milliseconds; 
        //syst.set_clock_source(SystClkSource::Core);
        // syst.set_reload(milliseconds/clock::get_nanoseconds()); // not sure how to access clock speed - it should be based on our clock.rs set_nanoseconds
        // syst.set_current(0);
        // syst.enable_interrupt();
        // syst.enable_counter(); // how does it necessarily execute the interrupt handler



    }
    fn get_period(&mut self, num: u8) -> Option<Word>{
            
            if num < 2 {
                return some(self.times[num])
            } else {
                return None;
            }

    }

    
    fn register_interrupt(&mut self, num: u8, func: &'a dyn FnMut(u8)) -> Result<(), ()>{
        self.handlers[Into::<usize>::into(num)] = func;
        Ok(())
    }
    
    // fn start(&mut self, num: u8){} // starts timer - watchout for timing errors

    // if specified timer has set period and handler
    //  set clock to start now
    //  when clock has finished period
    //      perform handler
    // else return error for starting a clock without period and handler
}

impl<'a> Timers<'a> for Arc<RwLock<TimersShim<'a>>> {
    

    fn register_interrupt(
        &mut self,
        num: u8,
        handler: &'a dyn Fn(pin: u8),
    ) -> Result<(), ())> {
        RwLock::write(self)
            .unwrap()
            .register_interrupt(num, handler)
    }
}


