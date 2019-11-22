extern crate futures; // 0.1.23
extern crate rand;
extern crate serde;
extern crate serde_json;
// use lc3_baseline_sim;
// use lc3_isa;
//use lc3_tui;
 use std::sync::mpsc::{Sender, Receiver};
use futures::{sync::mpsc, Async, Sink, Stream};
//use std::sync::mpsc;
// use std::{thread, time};
use bytes::{BytesMut, BufMut, BigEndian};
// use futures::future::{ ok};
 use serde::{Serialize, Deserialize};
use crate::error::Error;
use lc3_isa::*; 
use crate::control::Reg;
use crate::control::*;
 use {
     core::task::*,
     std::{
         future::Future,
         sync::{Arc, Mutex},
         sync::mpsc::{sync_channel, SyncSender},
         task::{Context, Poll},
         time::Duration,
     },
//     // The timer we wrote in the previous section:
//     //timer_future::TimerFuture,
 };

// Task executor that receives tasks off of a channel and runs them.
// struct Executor {
//     ready_queue: Receiver<Arc<Task>>,
// }

// /// `Spawner` spawns new futures onto the task channel.
// #[derive(Clone)]
// struct Spawner {
//     task_sender: SyncSender<Arc<Task>>,
// }

// /// A future that can reschedule itself to be polled by an `Executor`.
// struct Task {
//     /// In-progress future that should be pushed to completion.
//     ///
//     /// The `Mutex` is not necessary for correctness, since we only have
//     /// one thread executing tasks at once. However, Rust isn't smart
//     /// enough to know that `future` is only mutated from one thread,
//     /// so we need use the `Mutex` to prove thread-safety. A production
//     /// executor would not need this, and could use `UnsafeCell` instead.
//   //  future: Mutex<Option<BoxFuture<'static, ()>>>,

//     /// Handle to place the task itself back onto the task queue.
//     task_sender: SyncSender<Arc<Task>>,
// }

// fn new_executor_and_spawner() -> (Executor, Spawner) {
//     // Maximum number of tasks to allow queueing in the channel at once.
//     // This is just to make `sync_channel` happy, and wouldn't be present in
//     // a real executor.
//     const MAX_QUEUED_TASKS: usize = 10_000;
//     let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
//     (Executor { ready_queue }, Spawner { task_sender })
// }

// impl Spawner {
//     fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
//         let future = future.boxed();
//         let task = Arc::new(Task {
//             future: Mutex::new(Some(future)),
//             task_sender: self.task_sender.clone(),
//         });
//         self.task_sender.send(task).expect("too many tasks queued");
//     }
// }
//use crate::lc3_baseline_sim::sim;
// use std::time::Duration;
// use rand::distributions::{Range, IndependentSample};
// use core::future::Future;

static mut CURRENT_STATE: State = State::Paused;
use std::task::*;
#[derive(Serialize, Deserialize, Debug)]
enum device_status{
    PAUSE_SUCCESSFUL,
    STEP_SUCCESSFUL,
    RUN_COMPLETED,
    RUN_FAILED,
    STEP_UNSUCCESSFUL,
    PAUSE_UNSUCCESSFUL,

}
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

struct CurrentEvent{
  CurrentEvent: Event,
  CurrentState: State,

}
impl Future for CurrentEvent{

type Output = crate::control::Event;
fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output>{
           unsafe{ match CURRENT_STATE {
 //             let ret_status = device_status::PAUSE_SUCCESSFUL;

              Paused =>{
                CURRENT_STATE = State::Paused;
               Poll::Ready(Event::Interrupted)

              },

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
      }
    // Poll::Pending
    }
}

 struct device_control{

 }

  struct host_control{
    host_to_device_transmitter: Sender<std::string::String>,
    device_to_host_receiver:    Receiver<device_status>, 

 }
impl Control for host_control{
      // type EventFuture: Future<Output = Event>;
    type EventFuture = CurrentEvent;
   // type C = Self;
    fn get_pc(&self) -> Addr{
          let point = signal::GET_PC;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();   
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap()); 
         16 

    }
    fn set_pc(&mut self, addr: Addr){
          let point = signal::SET_PC(addr);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap(); 
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());     

    } // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word{
          let point = signal::GET_REGISTER(reg);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();   
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
         16


    }
    fn set_register(&mut self, reg: Reg, data: Word){

         let point = signal::SET_REGISTER(reg, data);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();  
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());

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
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
    }
    fn read_word(&self, addr: Addr) -> Word{
          let point = signal::READ_WORD(addr);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
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
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
        Ok(4)

    }
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>{
         let point = signal::UNSET_BREAKPOINT(idx);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
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
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
        Ok(16)

    }
    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>{
         let point = signal::UNSET_MEMORY_WATCH(idx);
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
        Ok(())

    }
    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES]{
         let point = signal::GET_MEMORY_WATCHES;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
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
          unsafe{ CURRENT_STATE = State::RunningUntilEvent;}
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
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
         unsafe{ CURRENT_STATE = State::RunningUntilEvent;}


    }
    fn pause(&mut self){
         let point = signal::PAUSE;
         let serialized = serde_json::to_string(&point).unwrap();
         println!("serialized = {}", serialized);
         //let (mut tx, mut rx) = self.channel1;
         self.host_to_device_transmitter.send(serialized).unwrap();
         println!("Output: {:?}", self.device_to_host_receiver.recv().unwrap());
         unsafe{ CURRENT_STATE = State::Paused;}

    }

    fn get_state(&self) -> State{
       unsafe{ CURRENT_STATE}

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




trait comm_channel: Control{
    type Transmitter_host;
    type Receiver_host;
    type Transmitter_Device;
    type Receiver_Device;
     type C: Control;
   //  host_to_device_transmitter: Sender<std::string::String>;
    // device_to_host_receiver:    Receiver<device_status>;
fn encode_and_transmit(&mut self){

}

fn decode_and_execute(&mut self, s: &str,sim:  &mut Self::C){

	            let mut dev_cpy = device_control{
                
            };
            let deserialized: signal = serde_json::from_str(s).unwrap();
             println!("Received deserialized: {:?}", deserialized);
             
             match deserialized{
                signal::RUN_UNTIL_EVENT => {println!("Issued and invoking Run until event");
                                            sim.run_until_event();

                                           //tx2.send(device_status::RUN_COMPLETED);

                                            },
                signal::SET_PC(addr)          => {
                                                  sim.set_pc(addr);

                                                 },
                signal::WRITE_WORD(word, value)     =>   {println!("Wrote word {:?} {:?}", word, value);
                                                          sim.write_word(word, value);

                                                         },
                signal::PAUSE        =>  {println!("Paused program");
                                          sim.pause();

                                         },
                signal::SET_BREAKPOINT(addr) => {println!("Set breakpoint");
                                                 sim.set_breakpoint(addr);
                                                },
                signal:: UNSET_BREAKPOINT(addr) => {println!("Unset breakpoint");
                                                    sim.unset_breakpoint(16);
                                                    },

                signal:: GET_BREAKPOINTS(idx) => {println!("Obtain breakpoints");
                                                  sim.get_breakpoints();

                                                  },
                signal:: GET_MAX_BREAKPOINTS => {
                                                println!("Max breakpoints");
                                               // sim.get_max_breakpoints();
                                                },
                signal:: SET_MEMORY_WATCH(addr)   => {
                                                      println!("Set memory watches" );
                                                      sim.set_memory_watch(16);
                                                     },
                signal:: UNSET_MEMORY_WATCH(idx) => {
                                                     println!("Unset memory watch" );
                                                     sim.unset_memory_watch(idx);
                                                    },
                signal:: GET_MAX_BREAKPOINTS => {
                                                println!("Get max break points");
                                                //get_max_breakpoints();
                                                },
                signal:: GET_MAX_MEMORY_WATCHES => {
                                                    println!("Get max breakpoints");
                                                   //sim.get_max_memory_watches();
                                                   },
                signal:: STEP                    => {
                                                    println!("Issue step");
                                                    sim.step();
                                                    },
                signal::GET_PC                 =>   {
                                                     println!("Get PC");
                                                     sim.get_pc();
                                                    }
                signal::READ_WORD(addr)            => {
                                                       println!("Read word");
                                                       sim.read_word(addr);
                                                      }
                signal::GET_MEMORY_WATCHES    =>     {
                                                      println!("Get memory watches");
                                                     sim.get_memory_watches();
                                                     }
                signal::SET_REGISTER(reg, word)    =>     {
                                                      println!("Get memory watches");
                                                      sim.set_register(reg, word);
                                                     }
                signal::GET_REGISTER(reg)    =>     {
                                                      println!("Get memory watches");
                                                      sim.get_register(reg);
                                                     }
             };	
}

}

// trait comm{}
// impl comm for comm_channel{

// }


#[derive(Serialize, Deserialize, Debug)]

enum Message {
  //  GetPc,
  //  GetPcReturnVal(Addr),
    //SetPc(u16),

  GET_PC ,
  GET_PC_RETURN_VAL(Addr),
  //GetPcReturnVal(Addr),
  SET_PC (u16),
  SET_PC_SUCCESS,
  WRITE_WORD (u16, u16),
  WRITE_WORD_SUCCESS,
  READ_WORD(u16),
  READ_WORD_RETURN_VAL(u16),
  PAUSE,
  PAUSE_SUCCESS,
  SET_BREAKPOINT (u16),
  SET_BREAKPOINT_SUCCESS,
  UNSET_BREAKPOINT (usize),
  UNSET_BREAKPOINT_SUCCESS,
  GET_BREAKPOINTS,
  GET_BREAKPOINTS_RETURN_VAL( [Option<Addr>; MAX_BREAKPOINTS]),
  GET_MAX_BREAKPOINTS,
  SET_MEMORY_WATCH (u16),
  SET_MEMORY_WATCH_SUCCESS,
  UNSET_MEMORY_WATCH (usize),
  UNSET_MEMORY_WATCH_SUCCESS,
  GET_MEMORY_WATCHES,
  GET_MEMORY_WATCHES_RETURN_VAL([Option<Addr>; MAX_MEMORY_WATCHES]),
  GET_MAX_MEMORY_WATCHES,
  STEP,
  STEP_SUCCESSFUL,
  RUN_UNTIL_EVENT,
  ISSUED_RUN_UNTIL_EVENT,
  SET_REGISTER(Reg, Word),
  SET_REGISTER_SUCCESS,
  GET_REGISTER(Reg),
  GET_REGISTER_RETURN_VAL(u16),
  COMMIT_MEMORY,
  COMMIT_MEMORY_SUCCESS,
}

trait TransportLayer {
    fn send(&self, message: Message) -> Result<(), ()>;

    fn get(&self) -> Option<Message>;
}

struct Server<T: TransportLayer> {
    transport: T,
}

impl<T: TransportLayer> Control for Server<T> {

     type EventFuture = CurrentEvent;
    fn get_pc(&self) -> Addr {
        let mut ret: u16 =0;
        self.transport.send(Message::GET_PC);

        if  let Some(m)=self.transport.get(){
        if let Message::GET_PC_RETURN_VAL(addr) = m {
           ret = addr;
        } 
        else{
            panic!();
        }
    }
    else{
        panic!();
    }
    ret
    //panic!();
}


  // fn set_pc(&mut self, addr: Addr){
        
  //       println!("Set PC to {:?}", (addr));
  //   } // Should be infallible.

    fn get_register(&self, reg: Reg) -> Word{

        let mut ret: u16 =0;
        self.transport.send(Message::GET_REGISTER(reg));

        if  let Some(m)=self.transport.get(){
        if let Message::GET_REGISTER_RETURN_VAL(val) = m {
           ret = val;
        } 
        else{
            panic!();
        }
    }
    else{
        panic!();
    }
    ret

    }
    fn set_register(&mut self, reg: Reg, data: Word){
        self.transport.send(Message::SET_REGISTER(reg, data));
        if  let Some(m)=self.transport.get(){
        if let Message::SET_REGISTER_SUCCESS = m {

        } 
        else{
            panic!();
        }
    }
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

        self.transport.send(Message::WRITE_WORD(addr, word));
        if  let Some(m)=self.transport.get(){
        if let Message::WRITE_WORD_SUCCESS = m {

        } 
        else{
            panic!();
        }
    }
    }
    fn read_word(&self, addr: Addr) -> Word{
        let mut ret: u16 =0;
        self.transport.send(Message::READ_WORD(addr));
        if  let Some(m)=self.transport.get(){
        if let Message::READ_WORD_RETURN_VAL(word) = m {
           ret = word;
        } 
        else{
            panic!();
        }
    }
    else{
        panic!();
    }
    ret

    }
    fn commit_memory(&self) -> Result<(), ()>{
        let mut res: Result<(), ()> = Err(());
        self.transport.send(Message::COMMIT_MEMORY);
        if  let Some(m)=self.transport.get(){
        if let Message::COMMIT_MEMORY_SUCCESS = m {
          res=  Ok(());
        } 
        else{
            res=Err(());
        }
    }
    res

    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>{
        let mut res: Result<usize, ()> = Err(());
        self.transport.send(Message::SET_BREAKPOINT(addr));
         if  let Some(m)=self.transport.get(){
        if let Message::SET_BREAKPOINT_SUCCESS = m {
           res= Ok(1)
        } 
        else{
           res=Err(());
        }
    }       
       res

    }
    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>{
        self.transport.send(Message::UNSET_BREAKPOINT(idx));
        //self.transport.send(Message::SET_MEMORY_WATCH(addr));
        let mut res: Result<(), ()> = Err(());
       // self.transport.send(Message::SET_MEMORY_WATCH(addr));
         if  let Some(m)=self.transport.get(){
        if let Message::UNSET_BREAKPOINT_SUCCESS = m {
           res= Ok(());
        } 
        else{
           res=Err(());
        }
    }       
       res

    }
    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS]{
        let mut ret: [Option<Addr>; MAX_BREAKPOINTS] = [None; MAX_BREAKPOINTS];
        self.transport.send(Message::GET_BREAKPOINTS);

        if  let Some(m)=self.transport.get(){
        if let Message::GET_BREAKPOINTS_RETURN_VAL(val) = m {
           ret = val;
        } 
        else{
            panic!();
        }
    }
    else{
        panic!();
    }
    ret
       // [None,None,None,None,None,None,None,None,None,None]

    }
    fn get_max_breakpoints() -> usize {
       // self.transport.send(Message::GET_REGISTER(reg));
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>{
        self.transport.send(Message::SET_MEMORY_WATCH(addr));
        let mut res: Result<usize, ()> = Err(());
       // self.transport.send(Message::SET_MEMORY_WATCH(addr));
         if  let Some(m)=self.transport.get(){
        if let Message::SET_MEMORY_WATCH_SUCCESS = m {
           res= Ok(1);
        } 
        else{
           res=Err(());
        }
    }       
       res

    }
    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>{
        self.transport.send(Message::UNSET_MEMORY_WATCH(idx));
        let mut res: Result<(), ()> = Err(());
       // self.transport.send(Message::SET_MEMORY_WATCH(addr));
         if  let Some(m)=self.transport.get(){
        if let Message::UNSET_MEMORY_WATCH_SUCCESS = m {
           res= Ok(());
        } 
        else{
           res=Err(());
        }
    }       
       res

    }
    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES]{
        let mut ret: [Option<Addr>; MAX_MEMORY_WATCHES] = [None; MAX_MEMORY_WATCHES];
        self.transport.send(Message::GET_MEMORY_WATCHES);

        if  let Some(m)=self.transport.get(){
        if let Message::GET_MEMORY_WATCHES_RETURN_VAL(val) = m {
           ret = val;
        } 
        else{
            panic!();
        }
    }
    else{
        panic!();
    }
    ret

    }
    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    // Execution control functions:
    fn run_until_event(&mut self)->Self::EventFuture {
         self.transport.send(Message::RUN_UNTIL_EVENT);
        if  let Some(m)=self.transport.get(){
        if let Message::ISSUED_RUN_UNTIL_EVENT = m {

        } 
        else{
            panic!();
        }
    }       
        unsafe{ CURRENT_STATE = State::RunningUntilEvent;}
        CurrentEvent{
            CurrentEvent: Event::Interrupted,
            CurrentState: State::RunningUntilEvent,
        }

    } // Can be interrupted by step or pause.
    fn step(&mut self){
        self.transport.send(Message::STEP);
        if  let Some(m)=self.transport.get(){
        if let Message::STEP_SUCCESSFUL = m {

        } 
        else{
            panic!();
        }
    }
    }
    fn pause(&mut self){
        self.transport.send(Message::PAUSE);
        if  let Some(m)=self.transport.get(){
        if let Message::PAUSE_SUCCESS = m {

        } 
        else{
            panic!();
        }
    }
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

        fn set_pc(&mut self, addr: Addr){
        self.transport.send(Message::SET_PC(addr));
        if  let Some(m)=self.transport.get(){
        if let Message::SET_PC_SUCCESS = m {

        } 
        else{
            panic!();
        }
    }
    } // Should be infallible.
}

struct Client<T: TransportLayer, C: Control> {
    transport: T,
    cont     : C,
}

struct MpscTransportLayer {}

// impl TransportLayer for MpscTransportLayer {
    
// }

impl<T: TransportLayer, C: Control>Client<T, C> {
    // Check for messages and execute them on something that implements the control
    // interface.

    // type EventFuture = CurrentEvent;
    // fn step(&mut self) {

    // }


  // fn set_pc(&mut self, addr: Addr){
        
  //       println!("Set PC to {:?}", (addr));
  //   } // Should be infallible.

  //   fn get_register(&self, reg: Reg) -> Word{
  //       16

  //   }
  //   fn set_register(&mut self, reg: Reg, data: Word){

  //   } // Should be infallible.

  //   fn get_registers_and_pc(&self) -> ([Word; 9], Word) {
  //       let mut regs = [0; 9];

  //       // use Reg::*;
  //       // [R0, R1, R2, R3, R4, R5, R6, R7, PSR]
  //       //     .iter()
  //       //     .enumerate()
  //       //     .for_each(|(idx, r)| regs[idx] = self.get_register(*r));

  //       (regs, self.get_pc())
  //   }

  //   fn write_word(&mut self, addr: Addr, word: Word){

  //   }
  //   fn read_word(&self, addr: Addr) -> Word{
  //       16

  //   }
  //   fn commit_memory(&self) -> Result<(), ()>{
  //       Ok(())

  //   }

  //   fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>{
  //       Ok(4)

  //   }
  //   fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>{
  //       Ok(())

  //   }
  //   fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS]{
  //       [None,None,None,None,None,None,None,None,None,None]

  //   }
  //   fn get_max_breakpoints() -> usize {
  //       MAX_BREAKPOINTS
  //   }

  //   fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>{
  //       Ok(16)

  //   }
  //   fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>{
  //       Ok(())

  //   }
  //   fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES]{
  //       [None,None,None,None,None,None,None,None,None,None]

  //   }
  //   fn get_max_memory_watches() -> usize {
  //       MAX_MEMORY_WATCHES
  //   }

  //   // Execution control functions:
  //   fn run_until_event(&mut self)->Self::EventFuture {
  //       CurrentEvent{
  //           CurrentEvent: Event::Interrupted,
  //           CurrentState: State::RunningUntilEvent,
  //       }

  //   } // Can be interrupted by step or pause.
  //   // fn step(&mut self){

  //   // }
  //   fn pause(&mut self){

  //   }

  //   fn get_state(&self) -> State{
  //       State::Paused

  //   }

  //   // TBD whether this is literally just an error for the last step or if it's the last error encountered.
  //   // If it's the latter, we should return the PC value when the error was encountered.
  //   //
  //   // Leaning towards it being the error in the last step though.
  //   fn get_error(&self) -> Option<Error>{
  //       None


  //   }


        fn step(&mut self) -> Addr{
        while let Some(m) = self.transport.get() {
            use Message::*;

            match m {
                GET_PC => {
                    self.transport.send(Message::GET_PC_RETURN_VAL(self.cont.get_pc()));
                }
                GET_PC_RETURN_VAL(h) => {},

                SET_PC(val) =>{
                    self.cont.set_pc(val);
                    self.transport.send(Message::SET_PC_SUCCESS);
                }

                // SET_MEMORY_WATCH(addr) =>{

                // }

                // UNSET_MEMORY_WATCH(idx) =>{

                // }

                SET_REGISTER(reg, word) =>{
                    self.cont.set_register(reg, word);
                    self.transport.send(Message::SET_REGISTER_SUCCESS);

                }

                RUN_UNTIL_EVENT => {//println!("Issued and invoking Run until event");
                                            self.cont.run_until_event();
                                            self.transport.send(Message::ISSUED_RUN_UNTIL_EVENT);
                                           //tx2.send(device_status::RUN_COMPLETED);

                                            },

                WRITE_WORD(word, value)     =>   {//println!("Wrote word {:?} {:?}", word, value);
                                                          self.cont.write_word(word, value);
                                                          self.transport.send(Message::WRITE_WORD_SUCCESS);

                                                         },
                PAUSE        =>  {println!("Paused program");
                                          self.cont.pause();

                                         },
                SET_BREAKPOINT(addr) => {       
                                                 println!("Set breakpoint");
                                                 self.cont.set_breakpoint(addr);
                                                },
                UNSET_BREAKPOINT(addr) => {println!("Unset breakpoint");
                                                    self.cont.unset_breakpoint(addr);
                                                    },

                GET_BREAKPOINTS => {println!("Obtain breakpoints");
                                                 let breaks = self.cont.get_breakpoints();
                                                 self.transport.send(Message::GET_BREAKPOINTS_RETURN_VAL(breaks));



                                                  },
                GET_MAX_BREAKPOINTS => {
                                                println!("Max breakpoints");
                                               // self.cont.get_max_breakpoints();
                                                },
                SET_MEMORY_WATCH(addr)   => {
                                                      println!("Set memory watches" );
                                                      self.cont.set_memory_watch(16);
                                                     },
                UNSET_MEMORY_WATCH(idx) => {
                                                     println!("Unset memory watch" );
                                                     self.cont.unset_memory_watch(idx);
                                                    },
                GET_MAX_BREAKPOINTS => {
                                                println!("Get max break points");
                                                //get_max_breakpoints();
                                                },
                GET_MAX_MEMORY_WATCHES => {
                                                    println!("Get max breakpoints");
                                                   //self.cont.get_max_memory_watches();
                                                   },
                STEP                    => {
                                                    println!("Issue step");
                                                    self.cont.step();
                                                    },

                READ_WORD(addr)           => {
                                                       println!("Read word");
                                                       self.cont.read_word(addr);
                                                      }
                GET_MEMORY_WATCHES    =>     {
                                                      println!("Get memory watches");
                                                     self.cont.get_memory_watches();
                                                     }

                GET_REGISTER(reg)    =>     {
                                                      println!("Get memory watches");
                                                      self.cont.get_register(reg);
                                                     }



                _=>{
                    panic!();
                },
            }
        }
        16

    }

    //     fn set_pc(&mut self, addr: Addr){
        
    //     println!("Set PC to {:?}", (addr));
    // } // Should be infallible.


    // `EventFuture`, `set_pc`, `get_register`, `set_register`, `write_word`, `read_word`, `commit_memory`, `set_breakpoint`, `unset_breakpoint`, `get_breakpoints`, `set_memory_watch`, `unset_memory_watch`, `get_memory_watches`, `run_until_event`, `step`, `pause`, `get_state`, `get_error`
}
