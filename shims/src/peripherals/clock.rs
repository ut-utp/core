use lc3_traits::peripherals::clock::Clock;
use lc3_isa::{Word, Addr, WORD_MAX_VAL};

use std::time::{Instant, SystemTime};
pub struct ClockShim {
    start_time: Instant
}

impl Default for ClockShim {
    fn default() -> Self {
        Self {
            start_time: Instant::now()
        }
    }
}

impl Clock for ClockShim {
    

  
    fn set_milliseconds(&mut self, ms: Word){
        
        

    } 


    fn get_milliseconds(&self) -> Word {
        (self.start_time.elapsed().as_millis() % (WORD_MAX_VAL as u128)).try_into().unwrap()
    }

}

