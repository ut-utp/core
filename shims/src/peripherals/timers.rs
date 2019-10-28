use lc3_traits::peripherals::timers::{Timers, TimerArr, TimerState, NUM_TIMERS};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
use core::ops::{Index, IndexMut};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
pub enum TimerEnum { T0, T1 }

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
pub struct TimersShim {
     states: TimerArr<TimerState>,
     times: TimerArr<Word>, 
     handlers: TimerArr<&'a dyn Fn(TimerEnum)>, // handlers for timers
}

// always access timers with u8, so no necessary indexes
const NO_OP: &dyn Fn(TimerEnum) = &|_| {};


impl Default for TimersShim<'_> {
    fn default() -> Self {
        Self {
            states: [TimerState::Disabled; NUM_TIMERS as usize],
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
    
         self.states[num] = state; 
       
        // match state {
        //     Repeated => TimerState::Repeated(true), // true? 
        //     SingleShot => State::SingleShot(true), // true?
        //     Disabled => State::Disabled,
        // };
        Ok(())
    }
    fn get_state(&mut self, num: u8) -> Option<TimerState>{ 
        // why is self mutable? Just getting...
        Some(self.states[num].into())

    }

    // fn singleShotTimer(&mut self, num: u8) -> Thread{
    //         return thread::spawn(|| {
    //             thread::sleep(Duration::from_millis(self.times[num]));
    //             self.handlers[num](num);
    //         });


    // }

    // fn repeatedTimer(&mut self, num: u8) -> Thread{
    //         let handle = thread::spawn(|| {
    //             loop {
    //                 thread::sleep(Duration::from_millis(self.times[num]));
    //                 self.handlers[num](num);
    //             }
    //         });

    //     return handle;
    // }


  fn set_period(&mut self, num: u8, milliseconds: Word){ 
      // thread based
        self.times[num] = milliseconds;
        let temp = thread::Builder::new(); 
        match self.states[num] {
            //Repeated => temp = self.repeatedTimer(num),
            //SingleShot => temp = self.singleShotTimer(num),
            Disabled => (),
        }


        // start a thread
        // set period = milliseconds
        // this means that for singleshot, we sleep for period
        // then execute func in handler

        // for repeated
        // start a thread, wait for period, and execute 


  }
      
    fn get_period(&mut self, num: u8) -> Option<Word>{
            
            if num < 2 {
                return Some(self.times[num])
            } else {
                return None;
            }

    }

    
    fn register_interrupt(&mut self, num: u8, func: &'a (dyn FnMut(u8) + Send)) -> Result<(), ()>{
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

