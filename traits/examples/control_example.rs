use core::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::{thread, time};

use lc3_isa::*;
use lc3_traits::control::Reg;
use lc3_traits::control::*;
use lc3_traits::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use lc3_traits::control_rpc::{Client, Message, Server, TransportLayer};
use lc3_traits::error::Error;

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
        16
    }

    fn set_pc(&mut self, addr: Addr) {
        println!("Set PC to {:?}", (addr));
    }

    fn get_register(&self, _reg: Reg) -> Word {
        16
    }

    fn set_register(&mut self, _reg: Reg, _data: Word) {}

    fn write_word(&mut self, _addr: Addr, _word: Word) {}
    fn read_word(&self, _addr: Addr) -> Word {
        16
    }
    fn commit_memory(&self) -> Result<(), ()> {
        Ok(())
    }

    fn set_breakpoint(&mut self, _addr: Addr) -> Result<usize, ()> {
        Ok(4)
    }

    fn unset_breakpoint(&mut self, _idx: usize) -> Result<(), ()> {
        Ok(())
    }

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        [None, None, None, None, None, None, None, None, None, None]
    }

    fn set_memory_watch(&mut self, _addr: Addr) -> Result<usize, ()> {
        Ok(16)
    }

    fn unset_memory_watch(&mut self, _idx: usize) -> Result<(), ()> {
        Ok(())
    }

    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES] {
        [None, None, None, None, None, None, None, None, None, None]
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {
        DummyDeviceCurrentEvent {
            current_event: Event::Interrupted,
            current_state: State::RunningUntilEvent,
        }
    } // Can be interrupted by step or pause.

    fn step(&mut self) {}
    fn pause(&mut self) {}

    fn get_state(&self) -> State {
        State::Paused
    }

    fn get_error(&self) -> Option<Error> {
        None
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
        server.set_memory_watch(15);
        server.unset_memory_watch(8);
        server.get_breakpoints();
        server.commit_memory();
        server.get_state();

        let one_sec = time::Duration::from_millis(1000);
        thread::sleep(one_sec);
    }
}
