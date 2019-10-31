use lc3_traits::peripherals::timers::{Timer, Timers, TimerArr, TimerState, TimerMiscError, TimerStateMismatch, NUM_TIMERS};

// timing errors occuring during scan cycles (input and ouput errors)
// errors handling overwriting handlers? Can timers have multiple handlers?
use lc3_isa::Word;
use std::thread;
use std::cell::Cell;
use std::sync::Mutex;
use std::time::Duration;


// The term “Single Shot” signifies a single pulse output of some duration. 
pub struct TimersShim {
     states: TimerArr<TimerState>,
     times: TimerArr<Option<Word>>, 
     handlers: TimerArr<&'static (dyn FnMut(Timer) + Send)>, // handlers for timers
}

const NO_OP: &(dyn FnMut(Timer) + Send) = &|_| {};

impl Default for TimersShim {
    fn default() -> Self {
        Self {
            states: [TimerState::Disabled; NUM_TIMERS as usize],
            times: [None; NUM_TIMERS as usize], // unlike gpio, interrupts occur on time - not on bit change
            handlers: [NO_OP; NUM_TIMERS as usize],
        }
    }
}



impl TimersShim{
     pub fn new() -> Self {
        Self::default()
    }
     fn singleshot_timer(&mut self, timer: Timer){
       
         let state_fixture = Mutex::new(Cell::new(self.times[usize::from(timer)]));
         
          
          thread::spawn(move || {
                let mut state_fixture = state_fixture.lock().unwrap();
                thread::sleep(Duration::from_millis((*state_fixture).get().unwrap() as u64)); 

             
          
            });
        

    }

    fn repeated_timer(&mut self, timer: Timer){
        
         let state_fixture = Mutex::new(Cell::new(self.times[usize::from(timer)]));
         let handle = thread::spawn(move || {
                loop {
                    let mut state_fixture = state_fixture.lock().unwrap();
                    thread::sleep(Duration::from_millis((*state_fixture).get().unwrap() as u64)); 
                }
            });


    }

}




impl Timers<'_> for TimersShim {
    fn set_state(&mut self, timer: Timer, state: TimerState) -> Result<(), ()>{
        use TimerState::*;
    
         self.states[usize::from(timer)] = state; 
       
        Ok(())
    }
    fn get_state(&self, timer: Timer) -> Option<TimerState> { 
        Some(self.states[usize::from(timer)].into())
    }

   

    fn set_period(&mut self, timer: Timer, milliseconds: Word){ 
      // thread based
        self.times[usize::from(timer)] = Some(milliseconds);
       // let temp = thread::Builder::new(); TODO: add return thread to kill repeated timers...
        use TimerState::*;
        match self.states[usize::from(timer)] {
            Repeated => self.repeated_timer(timer), 
            SingleShot => self.singleshot_timer(timer),
            Disabled => (),
        }

    }
      
    fn get_period(&self, timer: Timer) -> Option<Word> {
        self.times[usize::from(timer)]
    }

    fn register_interrupt(
        &mut self,
        timer: Timer,
        func: &'static (dyn FnMut(Timer) + Send)
    ) -> Result<(), ()> {
         self.handlers[usize::from(timer)] = func;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::timers::{Timer::*, Timers};

    #[test]
    fn get_disabled() {
        let shim = TimersShim::new();
        assert_eq!(shim.get_state(T0).unwrap(), TimerState::Disabled);
    }

    #[test]
     fn get_singleshot() {
        let mut shim = TimersShim::new();
        let res = shim.set_state(T0, TimerState::SingleShot);
        assert_eq!(shim.get_state(T0).unwrap(), TimerState::SingleShot);
    }

    #[test]
     fn get_repeated() {
        let mut shim = TimersShim::new();
        let res = shim.set_state(T0, TimerState::Repeated);
        assert_eq!(shim.get_state(T0).unwrap(), TimerState::Repeated);
    }

    #[test]
     fn get_set_period_singleshot() {
        let mut shim = TimersShim::new();
        let res = shim.set_state(T0, TimerState::SingleShot);
        shim.set_period(T0, 200);
        assert_eq!(shim.get_period(T0).unwrap(), 200);
    }


    #[test]
     fn get_set_period_repeated() {
        let mut shim = TimersShim::new();
        let res = shim.set_state(T0, TimerState::Repeated);
        shim.set_period(T0, 200);
        assert_eq!(shim.get_period(T0).unwrap(), 200);
    }

}
