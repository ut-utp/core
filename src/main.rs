extern crate futures; // 0.1.23
extern crate rand;
use futures::{sync::mpsc, Async, Sink, Stream};
//use std::sync::mpsc;
use std::{thread, time};
use prost::Message;
use bytes::{BytesMut, BufMut, BigEndian};
use futures::future::{ ok};
use serde::{Serialize, Deserialize};
use error::Error;
use prototype::*; 
use prototype::control::*;
use std::time::Duration;
use rand::distributions::{Range, IndependentSample};
use core::future::Future;
use std::task::*;

pub const MAX_BREAKPOINTS: usize = 10;
pub const MAX_MEMORY_WATCHES: usize = 10;
#[derive(Serialize, Deserialize, Debug)]
enum signal{
  GET_PC ,
  SET_PC (u16),
  WRITE_WORD (u16, u16),
  READ_WORD (u16),
  PAUSE,
  SET_BREAKPOINT (u16),
  UNSET_BREAKPOINT (usize),
  GET_BREAKPOINTS (u16),
  GET_MAX_BREAKPOINTS,
  SET_MEMORY_WATCH (u16),
  UNSET_MEMORY_WATCH (usize),
  GET_MEMORY_WATCHES,
  GET_MAX_MEMORY_WATCHES,
  STEP,
  RUN_UNTIL_EVENT,
}


#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
    message: signal,
}
#[derive(Serialize, Deserialize, Debug)]
struct set_pc{
    message: signal,
    addr: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_pc{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct write_word{
    message: signal,
    addr: u16,
    word: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct read_word{
    message: signal,
    addr: u16,
}


// struct write_word{
//     message: String,
//     addr: u16,
//     word: u16,
// }
#[derive(Serialize, Deserialize, Debug)]
struct set_breakpoint{
    message: signal,
    addr: u16,
}

// struct set_breakpoint{
//     message: String,
//     addr: u16,
// }
#[derive(Serialize, Deserialize, Debug)]
struct unset_breakpoint{
    message: signal,
    idx: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_breakpoints{
    message: signal,
    addr: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_max_breakpoints{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct set_memory_watch{
    message: signal,
    addr: u16,
}

// struct set_memory_watch{
//     message: signal,
//     addr: u16,
// }
#[derive(Serialize, Deserialize, Debug)]
struct unset_memory_watch{
    message: signal,
    idx: usize,
}

// struct get_memory_watches{
//     message: String,
// }

// struct get_max_memory_watches{
//     message: String,
// }

// struct unset_memory_watch{
//     message: String,
// }
#[derive(Serialize, Deserialize, Debug)]
struct step{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct pause{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_state{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct run_until_event{
    message: signal,
}
 use std::sync::mpsc::{Sender, Receiver};


 struct message_channel{
    channel1: (Sender<std::string::String>, Receiver<std::string::String>),
    channel2: (Sender<device_status>, Receiver<device_status>),


 }
// impl Future for host_control {
//     type Output = u16;


// }

 struct device_control{

 }

  struct host_control{
    host_to_device_transmitter: Sender<std::string::String>,
    device_to_host_receiver:    Receiver<device_status>, 

 }
// impl Future for signal{
//         type Output = Event;

//     fn poll(&mut self, ctx: &Context) -> Poll<Self::Output> {

//     }
// }
struct CurrentEvent{
  CurrentEvent: Event,
  CurrentState: State,

}
impl Future for CurrentEvent{

type Output=prototype::control::Event;
fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output>{
            match self.CurrentState {
 //             let ret_status = device_status::PAUSE_SUCCESSFUL;

              Paused => Poll::Ready(Event::Interrupted),
           //   2 => Poll::Ready(device_status::STEP_SUCCESSFUL),
           //   3 => Poll::Ready(device_status::PAUSE_SUCCESSFUL),
//         //     4 => Poll::Ready(device_status::RUN_FAILED),
//         //     5 => Poll::Ready(device_status::PAUSE_UNSUCCESSFUL),
//         //     6 => Poll::Ready(device_status::STEP_UNSUCCESSFUL),
              _ => {
                  cx.waker().clone().wake();//cx.waker().wake();
                  Poll::Pending
              }
          }
    // Poll::Pending
    }
}


impl Control for host_control{
      // type EventFuture: Future<Output = Event>;
    type EventFuture = CurrentEvent;

    fn get_pc(&self) -> Addr{
        16

    }
    fn set_pc(&mut self, addr: Addr){
          let point = signal::SET_PC(addr);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();      

    } // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word{
        16


    }
    fn set_register(&mut self, reg: Reg, data: Word){

    } // Should be infallible.

    fn get_registers_and_pc(&self) -> ([Word; 9], Word) {
        let mut regs = [0; 9];

        use Reg::*;
        [R0, R1, R2, R3, R4, R5, R6, R7, PSR]
            .iter()
            .enumerate()
            .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

        (regs, self.get_pc())
    }

    fn write_word(&mut self, addr: Addr, word: Word){
          let point = signal::WRITE_WORD(addr, word);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
    }
    fn read_word(&self, addr: Addr) -> Word{
          let point = signal::READ_WORD(addr);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
         16

    }
    fn commit_memory(&self) -> Result<(), ()>{
        Ok(())

    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>{
         let point = signal::SET_BREAKPOINT(addr);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
        Ok(4)

    }
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>{
         let point = signal::UNSET_BREAKPOINT(idx);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
        Ok(())

    }
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS]{
        //  let point = signal::GET_BREAKPOINTS;
        //  let serialized = serde_json::to_string(&point).unwrap();
        //  println!("serialized = {}", serialized);
        //  //let (mut tx, mut rx) = self.channel1;
        //  self.host_to_device_transmitter.send(serialized).unwrap();
         [None,None,None,None,None,None,None,None,None,None]

    }
    fn get_max_breakpoints() -> usize {
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>{
         let point = signal::SET_MEMORY_WATCH(addr);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
        Ok(16)

    }
    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>{
         let point = signal::UNSET_MEMORY_WATCH(idx);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
        Ok(())

    }
    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES]{
         let point = signal::GET_MEMORY_WATCHES;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
        [None,None,None,None,None,None,None,None,None,None]

    }
    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {
         let point = signal::RUN_UNTIL_EVENT;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
          println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());

        CurrentEvent{
            CurrentEvent: Event::Interrupted,
            CurrentState: State::RunningUntilEvent,
        }

    } // Can be interrupted by step or pause.
    fn step(&mut self){
         let point = signal::STEP;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();

    }
    fn pause(&mut self){
         let point = signal::PAUSE;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();

    }

    fn get_state(&self) -> State{
        State::Paused

    }

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error>{
        None


    }
}

impl Control for device_control{
      // type EventFuture: Future<Output = Event>;
   //   type EventFuture = Future<Output=Event>;
   type EventFuture = CurrentEvent;
    fn get_pc(&self) -> Addr{
        16

    }
    fn set_pc(&mut self, addr: Addr){
        
        println!("Set PC to {:?}", (addr));
    } // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word{
        16

    }
    fn set_register(&mut self, reg: Reg, data: Word){

    } // Should be infallible.

    fn get_registers_and_pc(&self) -> ([Word; 9], Word) {
        let mut regs = [0; 9];

        use Reg::*;
        [R0, R1, R2, R3, R4, R5, R6, R7, PSR]
            .iter()
            .enumerate()
            .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

        (regs, self.get_pc())
    }

    fn write_word(&mut self, addr: Addr, word: Word){

    }
    fn read_word(&self, addr: Addr) -> Word{
        16

    }
    fn commit_memory(&self) -> Result<(), ()>{
        Ok(())

    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>{
        Ok(4)

    }
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>{
        Ok(())

    }
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS]{
        [None,None,None,None,None,None,None,None,None,None]

    }
    fn get_max_breakpoints() -> usize {
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>{
        Ok(16)

    }
    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>{
        Ok(())

    }
    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES]{
        [None,None,None,None,None,None,None,None,None,None]

    }
    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    // Execution control functions:
    fn run_until_event(&mut self)->Self::EventFuture {
        CurrentEvent{
            CurrentEvent: Event::Interrupted,
            CurrentState: State::RunningUntilEvent,
        }

    } // Can be interrupted by step or pause.
    fn step(&mut self){

    }
    fn pause(&mut self){

    }

    fn get_state(&self) -> State{
        State::Paused

    }

    // TBD whether this is literally just an error for the last step or if it's the last error encountered.
    // If it's the latter, we should return the PC value when the error was encountered.
    //
    // Leaning towards it being the error in the last step though.
    fn get_error(&self) -> Option<Error>{
        None


    }
}




    // fn get_pc(&self) -> Addr;
    // fn set_pc(&mut self, addr: Addr); // Should be infallible.

    // fn get_register(&self, reg: Reg) -> Word;
    // fn set_register(&mut self, reg: Reg, data: Word); // Should be infallible.

    // fn write_word(&mut self, addr: Addr, word: Word);
    // fn read_word(&self, addr: Addr) -> Word;
    // fn commit_memory(&self) -> Result<(), ()>;

    // fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    // fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>;
    // fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS];


    // fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>;
    // fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>;
    // fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES];


    // // Execution control functions:
    // fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.
    // fn step(&mut self);
    // fn pause(&mut self);

    // fn get_state(&self) -> State;


        fn get_pc(tx:(Sender<std::string::String>)){
        let point = signal::GET_PC;
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        // let deserialized: get_pc = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
       // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
         tx.send(serialized).unwrap();
        // println!("got message: {}", receiver.recv().unwrap());
        }

         fn set_pc(tx:(Sender<std::string::String>)){
        let point = signal::SET_PC(500);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
        // println!("serialized = {}", serialized);
        // let deserialized: set_pc = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
         tx.send(serialized).unwrap();
        // println!("got message: {}", receiver.recv().unwrap());
        }

        fn step(tx:(Sender<std::string::String>)){
        //let point = step { message: signal::STEP};
        //let serialized = serde_json::to_string(&point).unwrap();
        //println!("serialized = {}", serialized);
        // let deserialized: set_pc = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
        //tx.send(serialized).unwrap();
      //  println!("got message: {}", receiver.recv().unwrap());
        }

        fn pause(tx:(Sender<std::string::String>)){
        //let point = pause { message: signal::PAUSE };
        //let serialized = serde_json::to_string(&point).unwrap();
        //println!("serialized = {}", serialized);
        // let deserialized: pause = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
        //tx.send(serialized).unwrap();
        //println!("got message: {}", receiver.recv().unwrap());
        }

        fn set_memory_watch(tx:(Sender<std::string::String>)){
        //let point = set_memory_watch { message: signal::SET_MEMORY_WATCH, addr: 2000 };
        //let serialized = serde_json::to_string(&point).unwrap();
       // println!("serialized = {}", serialized);
        // let deserialized: set_memory_watch = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
       // tx.send(serialized).unwrap();
       // println!("got message: {}", receiver.recv().unwrap());
        }

                fn set_breakpoint(tx:(Sender<std::string::String>)){
      //  let point = set_breakpoint { message: signal::SET_BREAKPOINT, addr: 2000 };
        //let serialized = serde_json::to_string(&point).unwrap();
       // println!("serialized = {}", serialized);
        // let deserialized: set_breakpoint = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
       // tx.send(serialized).unwrap();
       // println!("got message: {}", receiver.recv().unwrap());
        }


        fn unset_breakpoint(tx:(Sender<std::string::String>)){
        //let point = unset_breakpoint { message: signal::UNSET_BREAKPOINT, idx: 5 };
        //let serialized = serde_json::to_string(&point).unwrap();
        //println!("serialized = {}", serialized);
        // let deserialized: unset_breakpoint = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
        //tx.send(serialized).unwrap();
       // println!("got message: {}", receiver.recv().unwrap());
        }


             fn read_word(tx:(Sender<std::string::String>)){
        //let point = read_word { message: signal::READ_WORD, addr: 2000};
        //let serialized = serde_json::to_string(&point).unwrap();
        //println!("serialized = {}", serialized);
        // let deserialized: read_word = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
        //tx.send(serialized).unwrap();
        //println!("got message: {}", receiver.recv().unwrap());
        }


                fn unset_memory_watch(tx:(Sender<std::string::String>)){
        let point = signal::UNSET_MEMORY_WATCH(2000);
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        // let deserialized: unset_memory_watch = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
        tx.send(serialized).unwrap();
        //println!("got message: {}", receiver.recv().unwrap());
        }



                fn write_word(tx:(Sender<std::string::String>)){
        let point = signal::WRITE_WORD(2000, 1000);
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        // let deserialized: write_word = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
        tx.send(serialized).unwrap();
        //println!("got message: {}", receiver.recv().unwrap());
        }


        fn run_until_event( tx:(Sender<std::string::String>)){
        let point = signal::RUN_UNTIL_EVENT;
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);

        // let mut buf = BytesMut::with_capacity(1024);
        // let (sender, receiver) = std::sync::mpsc::channel();
         //println!("serialized = {:?}", serialized);
         tx.send(serialized).unwrap();
         let mut my_future = (Device_Signal::default());
         println!("Issued run until event: {:?}", run(my_future));
        // println!("got message:");
       // let deserialized: run_until_event = serde_json::from_str(&receiver.recv().unwrap()).unwrap();
        
         

        }

        // fn test_device_reception(){
        //     let (mut tx, rx) = mpsc::channel(1024);
        //         let handle = thread::spawn(move || {
        //     tx.send(1)
        //         .and_then(|tx| tx.send(2))
        //         .and_then(|tx| tx.send(3))
        //         .wait()
        //         .expect("Unable to send");
        // });

        // let mut rx = rx.map(|x| x * x);

        // handle.join().unwrap();

        // while let Ok(Async::Ready(Some(v))) = rx.poll() {
        //     println!("stream: {}", v);
        // }

        // }




// fn main() {
// 	    //let point = Point {message: signal::SET_PC, addr: 2000 };

//     // Convert the Point to a JSON string.
//     //let serialized = serde_json::to_string(&point).unwrap();

//     // Prints serialized = {"x":1,"y":2}
//     //println!("serialized = {}", serialized);

//     // Convert the JSON string back to a Point.
//     //let deserialized: Point = serde_json::from_str(&serialized).unwrap();

//     // Prints deserialized = Point { x: 1, y: 2 }
//     //println!("deserialized = {:?}", deserialized);
//     get_pc();
//     set_pc();
//     pause();
//     set_memory_watch();
//     unset_memory_watch();
//     write_word();
//     unset_breakpoint();
//     set_breakpoint();
//     read_word();
//     run_until_event();
//     test_device_reception();
//   //  let (tx, rx) = mpsc::channel(1000);
//       let temp:i32 = 1;
//   //  let mut xs: [i32; 5] = [1, 2, 3, 4, 5];



//     //println!("{}", a);
//   let (mut tx, rx) = mpsc::channel(1024);

//     thread::spawn(move || {
//         println!("--> START THREAD");
//         // We'll have the stream produce a series of values.
//         for _ in 0..10 {

//             let waited_for = sleep_temp();
//             println!("+++ THREAD WAITED {}", waited_for);

//             // When we `send()` a value it consumes the sender. Returning
//             // a 'new' sender which we have to handle. In this case we just
//             // re-assign.
//             match tx.send(waited_for).wait() {
//                 // Why do we need to do this? This is how back pressure is implemented.
//                 // When the buffer is full `wait()` will block.
//                 Ok(new_tx) => tx = new_tx,
//                 Err(_) => panic!("Oh no!"),
//             }

//         }
//         println!("<-- END THREAD");
//         // Here the stream is dropped.
//     });

//     // We can `.fold()` like we would an iterator. In fact we can do many
//     // things like we would an iterator.
//     let sum = rx.fold(0, |acc, val| {
//             // Notice when we run that this is happening after each item of
//             // the stream resolves, like an iterator.
//             println!("+++ FOLDING {} INTO {}", val, acc);
//             // `ok()` is a simple way to say "Yes this worked."
//             // `err()` also exists.
//             ok(acc + val)
//         })
//         .wait()
//         .unwrap();
//     println!("SUM {}", sum);
// }

use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut; 
thread_local!(static NOTIFY: RefCell<bool> = RefCell::new(true));

struct Context<'a> {
    waker: &'a Waker,
}

// const GLOBAL_STATE: Mutex<RefCell<bool>> = Mutex::nes

impl<'a> Context<'a> {
    fn from_waker(waker: &'a Waker) -> Self {
        Context { waker }
    }

    fn waker(&self) -> &'a Waker {
        &self.waker
    }
}

struct Waker;

#[derive(Serialize, Deserialize, Debug)]
enum device_status{
    PAUSE_SUCCESSFUL,
    STEP_SUCCESSFUL,
    RUN_COMPLETED,
    RUN_FAILED,
    STEP_UNSUCCESSFUL,
    PAUSE_UNSUCCESSFUL,

}
impl Waker {
    fn wake(&self) {
        NOTIFY.with(|f| *f.borrow_mut() = true)
    }
}

enum Poll2<T> {
    Ready(T),
    Pending,
}

trait Futur {
    type Output;

    fn poll(&mut self, cx: &Context) -> Poll2<Self::Output>;
}

#[derive(Default)]
struct Device_Signal {
    count: u32,
}
impl Deref for Device_Signal{
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.count
    }
}

impl DerefMut for Device_Signal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.count
    }
}

impl Futur for Device_Signal {
    type Output = device_status;

    fn poll(&mut self, ctx: &Context) -> Poll2<Self::Output> {
        match self.count {
           // let ret_status = device_status::PAUSE_SUCCESSFUL;

            1 => Poll2::Ready(device_status::RUN_COMPLETED),
            2 => Poll2::Ready(device_status::STEP_SUCCESSFUL),
            3 => Poll2::Ready(device_status::PAUSE_SUCCESSFUL),
            4 => Poll2::Ready(device_status::RUN_FAILED),
            5 => Poll2::Ready(device_status::PAUSE_UNSUCCESSFUL),
            6 => Poll2::Ready(device_status::STEP_UNSUCCESSFUL),
            _ => {
                ctx.waker().wake();
                Poll2::Pending
            }
        }
    }
}

fn run<F>(mut f: F) -> F::Output
where
    F: Futur,
{
    NOTIFY.with(|n| loop {
        if *n.borrow() {
            *n.borrow_mut() = false;
            let ctx = Context::from_waker(&Waker);
            if let Poll2::Ready(val) = f.poll(&ctx) {
                return val;

            }
        }
    })
}

// fn main() {
//    // let (sender, receiver) = std::sync::mpsc::channel();
//     let mut my_future = Device_Signal::default();

//     my_future.count = 4;
//     println!("Output: {:?}", run(my_future));
//     println!("Execution unblocked");
// }


static NTHREADS: usize = 1;
use std::sync::{Arc, Mutex};
fn main() {
    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
   
   // let (tx, rx): (Sender<std::string::String>, Receiver<std::string::String>) = std::sync::mpsc::channel();
   // let (tx2, rx2): (Sender<device_status>, Receiver<device_status>) = std::sync::mpsc::channel();
   // let mut ids2 = Vec::with_capacity(NTHREADS);
   let mut overall_channel = message_channel{
    channel1: std::sync::mpsc::channel(),
    channel2: std::sync::mpsc::channel(),
   };
   let (tx2, rx2) = overall_channel.channel2;
   let (tx, rx)   = overall_channel.channel1;

   let mut host_cpy = host_control{
    host_to_device_transmitter: tx,
    device_to_host_receiver:    rx2,
   };

        // The sender endpoint can be copied
       // let thread_tx = tx.clone();
    //let mut my_future = Arc::new((Device_Signal::default()));
    
   // let mutex = std::sync::Mutex::new(foo);
//let arc = std::sync::Arc::new(mutex);

   // my_future.count = 8;
        // Each thread will send its id via the channel

        thread::spawn(move || {
            let mut dev_cpy = device_control{
                
            };
            // The thread takes ownership over `thread_tx`
            // Each thread queues a message in the channel
           // thread_tx.send(3).unwrap();
         //  Arc::downgrade(&my_future);
          // (my_future).count=4;
          //let mut guard = my_future.lock().unwrap();
          //guard.count = 4;
           
             //tx.send(1.to_string());
             //let mut rx_set = Vec::new();
            // println!("Output: {:?}", rx.recv().wait().unwrap());
             loop{
            // let deserialized = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
            let deserialized: signal = serde_json::from_str(&rx.recv().unwrap()).unwrap();
             println!("Received deserialized: {:?}", deserialized);
             
             match deserialized{
                signal::RUN_UNTIL_EVENT => {println!("Issued and invoking Run until event");
                                            tx2.send(device_status::RUN_COMPLETED);

                                            },
                signal::SET_PC(addr)          => {
                                                  dev_cpy.set_pc(addr);

                                                 },
                signal::WRITE_WORD(word, value)     =>   {println!("Wrote word {:?} {:?}", word, value);
                                                          dev_cpy.write_word(word, value);

                                                         },
                signal::PAUSE        =>  {println!("Paused program");
                                          dev_cpy.pause();

                                         },
                signal::SET_BREAKPOINT(addr) => {println!("Set breakpoint");
                                                 dev_cpy.set_breakpoint(addr);
                                                },
                signal:: UNSET_BREAKPOINT(addr) => {println!("Unset breakpoint");
                                                    dev_cpy.unset_breakpoint(16);
                                                    },

                signal:: GET_BREAKPOINTS(idx) => {println!("Obtain breakpoints");
                                                  dev_cpy.get_breakpoints();

                                                  },
                signal:: GET_MAX_BREAKPOINTS => {
                                                println!("Max breakpoints");
                                                device_control::get_max_breakpoints();
                                                },
                signal:: SET_MEMORY_WATCH(addr)   => {
                                                      println!("Set memory watches" );
                                                      dev_cpy.set_memory_watch(16);
                                                     },
                signal:: UNSET_MEMORY_WATCH(idx) => {
                                                     println!("Unset memory watch" );
                                                     dev_cpy.unset_memory_watch(idx);
                                                    },
                signal:: GET_MAX_BREAKPOINTS => {
                                                println!("Get max break points");
                                                device_control::get_max_breakpoints();
                                                },
                signal:: GET_MAX_MEMORY_WATCHES => {
                                                    println!("Get max breakpoints");
                                                    device_control::get_max_memory_watches();
                                                   },
                signal:: STEP                    => {
                                                    println!("Issue step");
                                                    dev_cpy.step();
                                                    },
                signal::GET_PC                 =>   {
                                                     println!("Get PC");
                                                     dev_cpy.get_pc();
                                                    }
                signal::READ_WORD(addr)            => {
                                                       println!("Read word");
                                                       dev_cpy.read_word(addr);
                                                      }
                signal::GET_MEMORY_WATCHES    =>     {
                                                      println!("Get memory watches");
                                                      dev_cpy.get_memory_watches();
                                                     }
             };
             let one_sec = time::Duration::from_millis(1000);
             thread::sleep(one_sec);
         }
             
            // Sending is a non-blocking operation, the thread will continue
            // immediately after sending its message
           // println!("thread {} finished", id);
            //ids2.push(rx2.recv());
        });
        // thread::spawn(move ||{
    
        
        // } );
    //ISSUE: How is run_future now dereferenced
   // println!("Output: {:?}", run(my_future));

  //  thread::spawn(move|| {
            loop {
   // let tx = tx.clone();
       // set_pc(tx);
        host_cpy.set_pc(1000);
        host_cpy.set_pc(500);
        host_cpy.run_until_event();
        let one_sec = time::Duration::from_millis(1000);
        thread::sleep(one_sec);
        
    }
   // });

    println!("Output: {:?}", rx2.recv().unwrap());

    
    
        
    

    // Here, all the messages are collected
  //  let mut ids = Vec::with_capacity(NTHREADS);
 
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there no messages available
 

 

    // Show the order in which the messages were sent
    //println!("{:?}", ids);
}
pub fn sleep_temp() -> u64 {
    let mut generator = rand::thread_rng();
    let possibilities = Range::new(0, 100);

    let choice = possibilities.ind_sample(&mut generator);

    let a_little_bit = Duration::from_millis(choice);
    thread::sleep(a_little_bit);
    choice
}

