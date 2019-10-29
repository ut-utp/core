use lc3_traits::peripherals::timers::{Timer, Timers, TimerArr, TimerState, NUM_TIMERS};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
use core::ops::{Index, IndexMut};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

// #[derive(Copy, Clone, Debug)]
// pub enum State {
//     Repeated(bool), // need a way to remember which state timer is in
//     SingleShot(bool),
//    // ask why no interrupt state: Interrupt(bool),
//     Disabled,
// }



// impl From<State> for TimerState {
//     fn from(state: State) -> TimerState {
//         use TimerState::*;

//         match state {
//             State::Repeated(_) => Repeated,
//             State::SingleShot(_) => SingleShot, 
//             State::Disabled => Disabled,
//         }
//     }
// }

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
pub struct TimersShim<'a> {
     states: TimerArr<TimerState>,
     times: TimerArr<Option<Word>>, 
     handlers: TimerArr<&'a (dyn FnMut(Timer))>, // handlers for timers
}

const NO_OP: &(dyn FnMut(Timer) + Send) = &|_| {};

impl Default for TimersShim<'_> {
    fn default() -> Self {
        Self {
            states: [TimerState::Disabled; NUM_TIMERS as usize],
            times: [None; NUM_TIMERS as usize], // unlike gpio, interrupts occur on time - not on bit change
            handlers: [NO_OP; NUM_TIMERS as usize],
        }
    }
}

impl TimersShim<'_> {
     pub fn new() -> Self {
        Self::default()
    }
     fn singleShotTimer(&mut self, timer: Timer){
         // single thread wait and execute handler
          
          thread::scoped(move || {
        
                thread::sleep(Duration::from_millis(self.times[usize::from(timer)].unwrap() as u64));
           
              self.handlers[usize::from(timer)](timer);
          
            });
        

    }

    fn repeatedTimer(&mut self, timer: Timer){
        // loop through timer continuously 
            let handle = thread::scoped(move || {
                loop {
                    thread::sleep(Duration::from_millis(self.times[usize::from(timer)].unwrap() as u64));
                    self.handlers[usize::from(timer)](timer);
                }
            });

      //  return handle;
    }

}

impl<'a> Timers<'a> for TimersShim<'a> {
    fn set_state(&mut self, timer: Timer, state: TimerState) -> Result<(), ()>{
        use TimerState::*;
    
         self.states[usize::from(timer)] = state; 
       
        // match state {
        //     Repeated => TimerState::Repeated(true), // true? 
        //     SingleShot => State::SingleShot(true), // true?
        //     Disabled => State::Disabled,
        // };
        Ok(())
    }
    fn get_state(&self, timer: Timer) -> Option<TimerState> { 
        Some(self.states[usize::from(timer)].into())
    }

   

    fn set_period(&mut self, timer: Timer, milliseconds: Word){ 
      // thread based
        self.times[usize::from(timer)] = Some(milliseconds);
       // let temp = thread::Builder::new(); 
        use TimerState::*;
        match self.states[usize::from(timer)] {
            Repeated => self.repeatedTimer(timer),
            SingleShot => self.singleShotTimer(timer),
            Disabled => (),
            _ => (), // TODO: remove when other arms re-added
        }


        // start a thread
        // set period = milliseconds
        // this means that for singleshot, we sleep for period
        // then execute func in handler

        // for repeated
        // start a thread, wait for period, and execute 


    }
      
    fn get_period(&self, timer: Timer) -> Option<Word> {
        self.times[usize::from(timer)]
    }

    fn register_interrupt(
        &mut self,
        timer: Timer,
        func: &'a (dyn FnMut(Timer) + Send)
    ) -> Result<(), ()> {
        self.handlers[usize::from(timer)] = func;
        Ok(())
    }

    // fn start(&mut self, num: u8){} // starts timer - watchout for timing errors

    // if specified timer has set period and handler
    //  set clock to start now
    //  when clock has finished period
    //      perform handler
    // else return error for starting a clock without period and handler
}

