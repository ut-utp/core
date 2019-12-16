use core::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::{thread, time};

use lc3_isa::*;
use lc3_traits::control::control::{Control, Event, State, MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use lc3_traits::control::rpc::{Client, Message, MpscTransport, Server, TransportLayer};
use lc3_traits::error::Error;
use lc3_traits::memory::MemoryMiscError;
use lc3_traits::peripherals::{adc::*, gpio::*, pwm::*, timers::*};

use serde_json;

struct DummyDevice {}

struct DummyDeviceCurrentEvent {
    current_event: Event,
    current_state: State,
}

impl Future for DummyDeviceCurrentEvent {
    type Output = Event;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        use State::*;

        match self.current_state {
            Paused => Poll::Ready(Event::Interrupted),
            _ => {
                cx.waker().clone().wake();
                Poll::Pending
            }
        }
    }
}

impl Control for DummyDevice {
    type EventFuture = DummyDeviceCurrentEvent;

    fn get_pc(&self) -> Addr {
        0
    }

    fn get_register(&self, reg: Reg) -> Word {
        0
    }

    fn set_register(&mut self, reg: Reg, data: Word) {}

    fn write_word(&mut self, addr: Addr, word: Word) {}

    fn read_word(&self, addr: Addr) -> Word {
        0
    }

    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        Ok(())
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        Err(())
    }

    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        Err(())
    }

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        [None; MAX_BREAKPOINTS]
    }

    fn get_max_breakpoints() -> usize {
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr, data: Word) -> Result<usize, ()> {
        Err(())
    }

    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()> {
        Err(())
    }

    fn get_memory_watches(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] {
        [None; MAX_MEMORY_WATCHES]
    }

    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    fn run_until_event(&mut self) -> Self::EventFuture {
        DummyDeviceCurrentEvent {
            current_event: Event::Interrupted,
            current_state: State::RunningUntilEvent,
        }
    }

    fn step(&mut self) -> State {
        State::Paused
    }

    fn pause(&mut self) {}

    fn get_state(&self) -> State {
        State::Paused
    }

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error> {
        None
    }

    fn set_pc(&mut self, addr: Addr) { }
    fn get_gpio_states(&self) -> GpioPinArr<GpioState> {
        GpioPinArr::<GpioState>([GpioState::Input; GpioPin::NUM_PINS])
    }

    fn get_gpio_reading(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        GpioPinArr::<Result<bool, GpioReadError>>([Ok(false); GpioPin::NUM_PINS])
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState> {
        AdcPinArr::<AdcState>([AdcState::Enabled; AdcPin::NUM_PINS])
    }

    fn get_adc_reading(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
        AdcPinArr::<Result<u8, AdcReadError>>([Ok(4); AdcPin::NUM_PINS])
    }

    fn get_timer_states(&self) -> TimerArr<TimerState> {
        TimerArr::<TimerState>([TimerState::Repeated; TimerId::NUM_TIMERS])
    }

    fn get_timer_config(&self) -> TimerArr<Word> {
        TimerArr::<Word>([14; TimerId::NUM_TIMERS])
    }

    fn get_pwm_states(&self) -> PwmPinArr<PwmState> {
        PwmPinArr::<PwmState>([PwmState::Disabled; PwmPin::NUM_PINS])
    }

    fn get_pwm_config(&self) -> PwmPinArr<u8> {
        PwmPinArr::<u8>([4; PwmPin::NUM_PINS])
    }

    fn get_clock(&self) -> Word {
        16
    }

    fn reset(&mut self) {}
}

fn main() {
    let (host_channel, device_channel) = mpsc_transport_pair();

    let mut server = Server::<MpscTransport> {
        transport: host_channel,
    };

    let client = Client::<MpscTransport> {
        transport: device_channel,
    };

    let cl = Arc::new(Mutex::new(client));
    let counter = Arc::clone(&cl);

    let _handle = thread::spawn(move || {
        let mut dev_cpy = DummyDevice {};
        loop {
            (*counter).lock().unwrap().step(&mut dev_cpy);

            let one_sec = time::Duration::from_millis(1000);
            thread::sleep(one_sec);
        }
    });

    loop {
        // A sequence of 'commands' to demo the thing working:
        server.get_pc();
        server.step();
        server.read_word(40);
        server.write_word(0, 4);
        server.set_register(Reg::R4, 4);
        server.get_register(Reg::R4);
        server.pause();
        server.set_breakpoint(14);
        server.unset_breakpoint(14);
        server.set_memory_watch(15, 14);
        server.unset_memory_watch(8);
        server.get_breakpoints();
        server.commit_memory();
        server.get_state();

        let one_sec = time::Duration::from_millis(1000);
        thread::sleep(one_sec);
    }
}
