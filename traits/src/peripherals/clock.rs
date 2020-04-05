//! [`Clock` peripheral trait](Clock).

use crate::peripheral_trait;

use lc3_isa::Word;

// We've limited ourselves to only being able to count ~65 and a
// half seconds with our fancy clock (2 ^ 16 milliseconds).
// TODO: consider methods to increase the amount of time between rollovers.
// Options:
// - Decrease the precision of the clock to centiseconds
// - Increase the width of the clock to multiple words
// - Introduce more clocks
//   - A low-precision (s) *and* high-precision (ms) clock

/// A Clock peripheral for an LC-3 simulator.
///
/// Used for measuring lengths of time.
/// The clock provides getter and setter methods for a value
/// representing the time, in milliseconds, elapsed since its creation.
///
/// # Simple Definition
///
/// Put simply, the clock behaves as if it holds a word-length value which increments every millisecond.
/// The value starts at 0 and rolls back over to zero when its maximum value is reached.
/// The value can be retrieved or directly set with "get" and "set" methods, respectively.
/// After being set, the clock continues incrementing from the value that was set.
/// For a more general definition, see below.
///
/// # General Definition
///
/// For the millisecond after creation, getting the value of the clock returns 0.
/// In general, getting the value of the clock returns the number of milliseconds since creation.
/// However, the value of the clock must be word-length. When the value of the clock reaches the maximum
/// value representable in a word, it will "roll over" in the next millisecond, resetting to 0.
/// So, more precisely, getting the value of the clock returns the number of milliseconds since creation
/// modulo the maximum value representable in a word.
///
/// Setting the value of the clock should take a word,
/// then cause subsequent calls to get the value to return that word plus the number of
/// milliseconds since creation, all modulo the maximum value representable in a word.
/// In other words, the clock will "restart" with its value at the given word
/// and continue keeping track of the milliseconds and rolling over.
///
/// # Reasoning
///
/// We provide the Clock peripheral to enable simulator users to measure lengths of time.
/// By getting the clock's value twice within the rollover period, users can accurately
/// measure (relatively short) time spans.
///
/// The purpose of the Clock peripheral is distinct from the Timers peripheral in that
/// timers do not provide the amount of time they have been running, and so don't enable
/// measuring time. While both deal with time, timers are useful for scheduling events
/// at known periods or lengths of time, while the clock is useful for measuring lengths
/// of time that events take.
///
peripheral_trait! {clock,
pub trait Clock: Default {
    fn get_milliseconds(&self) -> Word;

    fn set_milliseconds(&mut self, ms: Word);
}}

// TODO: roll this into the macro
using_std! {
    use std::sync::{Arc, RwLock};
    impl<C: Clock> Clock for Arc<RwLock<C>> {
        fn get_milliseconds(&self) -> Word {
            RwLock::read(self).unwrap().get_milliseconds()
        }

        fn set_milliseconds(&mut self, ms: Word) {
            RwLock::write(self).unwrap().set_milliseconds(ms)
        }
    }
}
