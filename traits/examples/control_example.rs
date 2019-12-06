use core::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::{thread, time};

use lc3_traits::memory::MemoryMiscError;
use lc3_isa::*;
//use lc3_isa::Reg;
use lc3_traits::control::*;
use lc3_traits::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use lc3_traits::control_rpc::{Client, Message, Server, TransportLayer};
use lc3_traits::error::Error;
use lc3_traits::peripherals::{gpio::*, timers::*, pwm::*, adc::*};

use serde_json;

struct DummyDevice {}

struct DummyDeviceCurrentEvent {
    current_event: Event,
    current_state: State,
}

impl Future for DummyDeviceCurrentEvent {
    type Output = lc3_traits::control::Event;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        use lc3_traits::control::State::*;

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
        let mut ret: u16 = 0;

        ret
        //panic!();
    }

    fn get_register(&self, reg: Reg) -> Word {
        let mut ret: u16 = 0;

        ret
    }

    fn set_register(&mut self, reg: Reg, data: Word) {

    } // Should be infallible.

    // fn get_registers_and_pc(&self) -> ([Word; 9], Word); // TODO

    fn write_word(&mut self, addr: Addr, word: Word) {

    }

    fn read_word(&self, addr: Addr) -> Word {
        let mut ret: u16 = 0;

        ret
    }

    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        //let mut res: Result<(), MemoryMiscError>;// = Err(());
        Ok(())
       // res
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        let mut res: Result<usize, ()> = Err(());

        res
    }

    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
      //  self.transport.send(Message::UNSET_BREAKPOINT(idx));
        //self.transport.send(Message::SET_MEMORY_WATCH(addr));
        let mut res: Result<(), ()> = Err(());
        // self.transport.send(Message::SET_MEMORY_WATCH(addr));

        res
    }

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        let mut ret: [Option<(Addr)>; MAX_BREAKPOINTS] = [None; MAX_BREAKPOINTS];

        ret
    }

    fn get_max_breakpoints() -> usize {
        // TODO: Actually proxy this
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr, data: Word) -> Result<usize, ()> {
       // self.transport.send(Message::SET_MEMORY_WATCH(addr, data));
        let mut res: Result<usize, ()> = Err(());

        res
    }

    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()> {
       // self.transport.send(Message::UNSET_MEMORY_WATCH(idx));
        let mut res: Result<(), ()> = Err(());

        
        res
    }

    fn get_memory_watches(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] {
        let mut ret: [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] = [None; MAX_MEMORY_WATCHES];

        ret
    }

    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {

        DummyDeviceCurrentEvent {
            current_event: Event::Interrupted,
            current_state: State::RunningUntilEvent,
        }
    } // Can be interrupted by step or pause.
    fn step(&mut self) -> State {
        let mut ret: State = State::Paused;

        ret
    }
    fn pause(&mut self) {

    }

    fn get_state(&self) -> State {
        let mut ret: State = State::Paused;

        ret
    }

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error> {
        None
    }

    fn set_pc(&mut self, addr: Addr) {
        //self.transport.send(Message::SET_PC(addr));
    } // Should be infallible.
    fn get_gpio_states(&self) -> GpioPinArr<GpioState>{
        GpioPinArr::<GpioState>([GpioState::Input; GpioPin::NUM_PINS])// = State::Paused;
        //ret
    }

    fn get_gpio_reading(&self) -> GpioPinArr<Result<bool, GpioReadError>>{
    GpioPinArr::<Result<bool, GpioReadError>>([Ok((false)); GpioPin::NUM_PINS])
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState>{
      //  let mut ret: AdcPinArr<AdcState>;
      AdcPinArr::<AdcState>([AdcState::Enabled;AdcPin::NUM_PINS] )
        //ret


    }
    fn get_adc_reading(&self) -> AdcPinArr<Result<u8, AdcReadError>>{
         AdcPinArr::<Result<u8, AdcReadError>>([Ok((4));AdcPin::NUM_PINS] )// = State::Paused;
         //ret
    }
    fn get_timer_states(&self) -> TimerArr<TimerState>{
            TimerArr::<TimerState>([TimerState::Repeated;TimerId::NUM_TIMERS])

    }
    fn get_timer_config(&self) -> TimerArr<Word>{
        TimerArr::<Word>([14;TimerId::NUM_TIMERS])

    }
    fn get_pwm_states(&self) -> PwmPinArr<PwmState>{
       PwmPinArr::<PwmState>([PwmState::Disabled; PwmPin::NUM_PINS])

    }
    fn get_pwm_config(&self) -> PwmPinArr<u8>{
       PwmPinArr::<u8>([4; PwmPin::NUM_PINS])
    }
    fn get_clock(&self) -> Word{
        16

    }
    fn reset(&mut self){

    }
}

struct MpscTransport {
    tx: Sender<std::string::String>,
    rx: Receiver<std::string::String>,
}

impl TransportLayer for MpscTransport {
    fn send(&self, message: Message) -> Result<(), ()> {
        let point = message;
        let serialized = serde_json::to_string(&point).unwrap();

        self.tx.send(serialized).unwrap();

        Ok(())
    }

    fn get(&self) -> Option<Message> {
        let deserialized: Message = serde_json::from_str(&self.rx.recv().unwrap()).unwrap();

        println!("deserialized = {:?}", deserialized);
        Some(deserialized)
    }
}

fn mpsc_transport_pair() -> (MpscTransport, MpscTransport) {
    let (tx_h, rx_h) = std::sync::mpsc::channel();
    let (tx_d, rx_d) = std::sync::mpsc::channel();

    let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
    let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

    (host_channel, device_channel)
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
