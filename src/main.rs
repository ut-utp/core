//extern crate futures; // 0.1.23
extern crate rand;
extern crate serde;
extern crate serde_json;
//use lc3_traits;
//use lc3_baseline_sim;
//use lc3_isa;
//use lc3_tui;
//use futures::{sync::mpsc, Async, Sink, Stream};
//use std::sync::mpsc;
use std::{thread, time};
//use prost::Message;
use bytes::{BytesMut, BufMut, BigEndian};
//use futures::future::{ ok};
use serde::{Serialize, Deserialize};
use lc3_traits::error::Error;
use lc3_isa::*; 
use lc3_traits::control::Reg;
use lc3_traits::control::*;
use lc3_traits::transport_layer::{Message, TransportLayer, Server, Client};
use std::time::Duration;
use rand::distributions::{Range, IndependentSample};
use core::future::Future;
use std::task::{Context, Poll, Waker};
 use std::sync::mpsc::{sync_channel, SyncSender, Sender, Receiver};
//use futures::future::*;
use {
    futures::{
        future::{FutureExt, BoxFuture},
        task::{ArcWake, waker_ref},
    },
};
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
  SET_REGISTER(Reg, Word),
  GET_REGISTER(Reg),

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

struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<std::task::Waker>,
}

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct CurrentEvent{
  CurrentEvent: Event,
  CurrentState: State,
  //shared_state: Arc<Mutex<SharedState>>,

}
impl Future for CurrentEvent{

type Output=lc3_traits::control::Event;
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


impl Future for TimerFuture {
    type Output = lc3_traits::control::Event;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(Event::Interrupted)
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}


impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Spawn the new thread
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // Signal that the timer has completed and wake up the last
            // task on which the future was polled, if one exists.
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}

/// Task executor that receives tasks off of a channel and runs them.
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("too many tasks queued");
    }
}

impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if let Poll::Pending = future.as_mut().poll(context) {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
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
          let point = signal::GET_REGISTER(reg);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();   
         16


    }
    fn set_register(&mut self, reg: Reg, data: Word){

         let point = signal::SET_REGISTER(reg, data);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();  

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
    let (executor, spawner) = new_executor_and_spawner();

    // Spawn a task to print before and after waiting on a timer.
    spawner.spawn(async {
        println!("howdy!");
        // Wait for our timer future to complete after two seconds.
        TimerFuture::new(Duration::new(1, 0)).await;
        println!("done!");
    });

    // Drop the spawner so that our executor knows it is finished and won't
    // receive more incoming tasks to run.
    drop(spawner);

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    executor.run();
        CurrentEvent{
            CurrentEvent: Event::Interrupted,
            CurrentState: State::RunningUntilEvent,
            //shared_state:()
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

        // use Reg::*;
        // [R0, R1, R2, R3, R4, R5, R6, R7, PSR]
        //     .iter()
        //     .enumerate()
        //     .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

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
            //shared_state:()
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
         //tx.send(serialized).unwrap();
        // let mut my_future = (Device_Signal::default());
        // println!("Issued run until event: {:?}", run(my_future));
        // println!("got message:");
       // let deserialized: run_until_event = serde_json::from_str(&receiver.recv().unwrap()).unwrap();
        
         

        }


#[derive(Serialize, Deserialize, Debug)]
enum device_status{
    PAUSE_SUCCESSFUL,
    STEP_SUCCESSFUL,
    RUN_COMPLETED,
    RUN_FAILED,
    STEP_UNSUCCESSFUL,
    PAUSE_UNSUCCESSFUL,

}


#[derive(Default)]
struct Device_Signal {
    count: u32,
}



static NTHREADS: usize = 1;
use std::sync::{Arc, Mutex};


struct MPSC_Transport{
    tx: Sender<std::string::String>,
    rx: Receiver<std::string::String>,
   // channel2: (Sender<Message>, Receiver<Message>),
}

impl TransportLayer for MPSC_Transport{
   fn send(&self, message: Message) -> Result<(), ()>{
        let point = message;
        let serialized = serde_json::to_string(&point).unwrap();
        //let (tx, rx)= &self.channel;
        self.tx.send(serialized).unwrap();

       // self.port.write(serialized);
    Ok(())

   }
   
   fn get(&self) -> Option<Message>{
    //let deserialized: run_until_event = serde_json::from_str(&receiver.recv().unwrap()).unwrap();
    //let (tx, rx)= &self.channel;
    let deserialized: Message = serde_json::from_str(&self.rx.recv().unwrap()).unwrap();
    println!("deserialized = {:?}", deserialized);
    Some(deserialized)
   }
}

fn main() {
    
    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
   
   // let (tx, rx): (Sender<std::string::String>, Receiver<std::string::String>) = std::sync::mpsc::channel();
   // let (tx2, rx2): (Sender<device_status>, Receiver<device_status>) = std::sync::mpsc::channel();
   // let mut ids2 = Vec::with_capacity(NTHREADS);
   let channel1= std::sync::mpsc::channel();
   let channel2= std::sync::mpsc::channel();
   let (tx_h, rx_h) = channel1;
   let (tx_d, rx_d) = channel2;

   let mut HostChannel = MPSC_Transport{
    //channel: std::sync::mpsc::channel(),
    tx: tx_h,
    rx: rx_d,
   };
   let mut DeviceChannel = MPSC_Transport{
   // channel2: std::sync::mpsc::channel(),
   tx: tx_d,
   rx: rx_h,
    //channel1: std::sync::mpsc::channel(),
   };

   let mut server = Server::<MPSC_Transport>{
        transport: HostChannel,
   };

   let client = Client::<MPSC_Transport>{
        transport: DeviceChannel,
   };
   let cl = Arc::new(Mutex::new(client));
   //let (tx2, rx2) = server.transport.channel2;
   //let (tx, rx)   = server.transport.channel1;

   // let mut host_cpy = host_control{
   //  //host_to_device_transmitter: tx,
   //  //device_to_host_receiver:    rx2,
   // };
    let counter = Arc::clone(&cl);
    
      let handle =   thread::spawn(move || {
       // let counter = Arc::clone(&serv);
             let mut dev_cpy = device_control{
                
             };
             loop{
            // let deserialized = serde_json::from_str(&serialized).unwrap();
        // println!("deserialized = {:?}", deserialized);
         //let deserialized: signal = serde_json::from_str(&rx.recv().unwrap()).unwrap();
             //println!("Received deserialized: {:?}", deserialized);
              (*counter).lock().unwrap().step(&mut dev_cpy);
             // println!("came here");
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
       // host_cpy.set_pc(1000);
       // host_cpy.set_pc(500);
       // host_cpy.run_until_event();
        server.get_pc();
        server.step();
        server.read_word(40);
        server.write_word( 0, 4);
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
    handle.join().unwrap();
   // });

  //  println!("Output: {:?}", rx2.recv().unwrap());

    
    
        
    

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

