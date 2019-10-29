use lc3_traits::peripherals::clock::Clock;
use lc3_isa::{Word, Addr, WORD_MAX_VAL};
use core::convert::TryInto;
use std::ops::Add;

use std::time::{Instant, SystemTime, Duration};
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
    
    fn get_milliseconds(&self) -> Word {
        (self.start_time.elapsed().as_millis() % (WORD_MAX_VAL as u128))
            .try_into()
            .unwrap()
    }
        // they set milliseconds - adding to the current time,
        // next time that they call get_milliseconds(), 
        // they will get the input milliseconds
        fn set_milliseconds(&mut self, ms: Word){
        self.start_time = self.start_time.add(Duration::from_millis(ms as u64));
     
    }

}

