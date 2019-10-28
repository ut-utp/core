
use crate::peripherals::clock::Clock;
use std::time::{Instant, SystemTime};
pub struct ClockShim {
    start_time: Instant
}

impl Default for ClockShim {
    fn default() -> Self {
        Self {
            start_time: Instant::now();
        }
    }
}

impl Clock for ClockShim {
    

    // isn't milliseconds too large?
    // shouldn't it be more like nano, because PLL can generate
    // frequencies between 3.12MHz to 80MHz - TExaS_Init sets at 80MHz from what I remember
    fn set_milliseconds(&mut self, ms: Word){
        
    } // want to be able to set to 80MHz, requiring 12.5 nano seconds 


    fn get_nanoseconds(&self) -> Word{
    (self.start_time.elapsed().as_millis() % (WORD_MAX_VAL as u128).try_into().unwrap(); 
    }

}

