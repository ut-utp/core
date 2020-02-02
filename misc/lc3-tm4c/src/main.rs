// #![cfg_attr(not(test), no_std)]
#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger


mod memory;
mod peripherals;
// mod transport;

use memory::Tm4cMemory;
use peripherals::{Tm4cGpio, Tm4cAdc, Tm4cPwm, Tm4cPwm, Tm4cTimers, Tm4cClock, Tm4cInput, Tm4cOutput};

use cortex_m::asm;
use cortex_m_rt::entry;

use lc3_traits::control::rpc::SimpleEventFutureSharedState;

// static SHARED_STATE: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

use lc3_baseline_sim::interp::{Interpreter, InstructionInterpreter, InterpreterBuilder, PeripheralInterruptFlags};
use lc3_baseline_sim::sim::Simulator;

use lc3_traits::peripherals::PeripheralSet;
use lc3_traits::control::Control;

#[entry]
fn main() -> ! {
    type Interp<'a> = Interpreter<'a,
        Tm4cMemory,
        PeripheralSet<'a,
            Tm4cGpio<'a>,
            Tm4cAdc,
            Tm4cPwm,
            Tm4cTimers<'a>,
            Tm4cClock,
            Tm4cInput<'a>,
            Tm4cOutput<'a>,
        >
    >;

    let flags: PeripheralInterruptFlags = PeripheralInterruptFlags::new();
    let state: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

    let mut interp: Interp = InterpreterBuilder::new()
        .with_defaults()
        .build();

    interp.reset();
    interp.init(&flags);

    let mut sim = Simulator::new_with_state(interp, &state);
    sim.reset();

    loop {
        // your code goes here
        sim.step();
    }
}
