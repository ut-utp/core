use lc3_traits::peripherals::clock::Clock;
use lc3_isa::{Word, Addr, WORD_MAX_VAL};
use core::convert::TryInto;
use std::ops::{Sub, Add};
use std::thread::sleep;

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
          let time = Duration::from_millis(ms as u64);
          self.start_time = self.start_time.checked_sub(time).unwrap();
          
            
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use lc3_traits::peripherals::clock::Clock;

    #[test]
    fn test_get_milliseconds() {
        let clock = ClockShim::default();
        let now = Instant::now();
        sleep(Duration::from_millis(1000));
        assert_eq!(clock.get_milliseconds(), (now.elapsed().as_millis() % (WORD_MAX_VAL as u128)) as u16)
    }
    #[test]
    fn test_get_milliseconds_wrong() {
         let clock = ClockShim::default();
        sleep(Duration::from_millis(1000));
         let now = Instant::now();
        assert_ne!(clock.get_milliseconds(), (now.elapsed().as_millis() % (WORD_MAX_VAL as u128)) as u16)
    }
    
    #[test]
    fn test_set_milliseconds() {
        let mut clock = ClockShim::default();
        let now = Instant::now();
        clock.set_milliseconds(100);
         assert_eq!(clock.get_milliseconds(), 100 + ((now.elapsed().as_millis() % (WORD_MAX_VAL as u128)) as u16))

    }

        #[test]
    fn test_set_milliseconds_wrong() {
        let mut clock = ClockShim::default();
        let now = Instant::now();
        clock.set_milliseconds(1000);
         assert_ne!(clock.get_milliseconds(), 100 + ((now.elapsed().as_millis() % (WORD_MAX_VAL as u128)) as u16))

    }


}
