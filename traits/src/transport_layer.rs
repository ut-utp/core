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

static mut CURRENT_STATE: State = State::Paused;
use std::task::*;
pub struct CurrentEvent{
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
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
   GET_STATE,
   GET_STATE_RETURN_VAL(State),
}

pub trait TransportLayer {
   fn send(&self, message: Message) -> Result<(), ()>;
   
   fn get(&self) -> Option<Message>;
}

pub struct Server<T: TransportLayer> {
   pub transport: T,
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
       let mut ret: State =State::Paused;
      self.transport.send(Message::GET_STATE);
      
      if  let Some(m)=self.transport.get(){
         if let Message::GET_STATE_RETURN_VAL(addr) = m {
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

pub struct Client<T: TransportLayer> {
   pub transport: T,
}

struct MpscTransportLayer {}

// impl TransportLayer for MpscTransportLayer {

// }

impl<T: TransportLayer>Client<T> {
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
   
   
   pub fn step<C: Control>(&mut self, cont: &mut C) -> Addr{
      while let Some(m) = self.transport.get() {
         use Message::*;
         
         match m {
            GET_PC => {
               self.transport.send(Message::GET_PC_RETURN_VAL(cont.get_pc()));
            }
            GET_PC_RETURN_VAL(h) => {},
            
            SET_PC(val) =>{
               cont.set_pc(val);
               self.transport.send(Message::SET_PC_SUCCESS);
            }
            
            // SET_MEMORY_WATCH(addr) =>{
            
            // }
            
            // UNSET_MEMORY_WATCH(idx) =>{
            
            // }
            
            SET_REGISTER(reg, word) =>{
               cont.set_register(reg, word);
               self.transport.send(Message::SET_REGISTER_SUCCESS);
               
            }
            
            RUN_UNTIL_EVENT => {//println!("Issued and invoking Run until event");
               cont.run_until_event();
               self.transport.send(Message::ISSUED_RUN_UNTIL_EVENT);
               //tx2.send(device_status::RUN_COMPLETED);
               
            },
            
            WRITE_WORD(word, value)     =>   {//println!("Wrote word {:?} {:?}", word, value);
               cont.write_word(word, value);
               self.transport.send(Message::WRITE_WORD_SUCCESS);
               
            },
            PAUSE        =>  {println!("Paused program");
               cont.pause();
               self.transport.send(Message::PAUSE_SUCCESS);
               
            },
            SET_BREAKPOINT(addr) => {
               println!("Set breakpoint");
               cont.set_breakpoint(addr);
               self.transport.send(Message::SET_BREAKPOINT_SUCCESS);
            },
            UNSET_BREAKPOINT(addr) => {println!("Unset breakpoint");
               cont.unset_breakpoint(addr);
               self.transport.send(UNSET_BREAKPOINT_SUCCESS);
            },
            
            GET_BREAKPOINTS => {println!("Obtain breakpoints");
               let breaks = cont.get_breakpoints();
               self.transport.send(Message::GET_BREAKPOINTS_RETURN_VAL(breaks));
               
               
               
            },
            GET_MAX_BREAKPOINTS => {
               println!("Max breakpoints");
               // self.cont.get_max_breakpoints();
            },
            SET_MEMORY_WATCH(addr)   => {
               println!("Set memory watches" );
               cont.set_memory_watch(16);
               self.transport.send(Message::SET_MEMORY_WATCH_SUCCESS);
            },
            UNSET_MEMORY_WATCH(idx) => {
               println!("Unset memory watch" );
               cont.unset_memory_watch(idx);
               self.transport.send(Message::UNSET_MEMORY_WATCH_SUCCESS);
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
               cont.step();
               self.transport.send(Message::STEP_SUCCESSFUL);
            },
            
            READ_WORD(addr)           => {
               println!("Read word");
              self.transport.send(Message::READ_WORD_RETURN_VAL( cont.read_word(addr)));

            }
            GET_MEMORY_WATCHES    =>     {
               println!("Get memory watches");
               cont.get_memory_watches();
            }
            
            GET_REGISTER(reg)    =>     {
               println!("Get Registers");
               
               self.transport.send(Message::GET_REGISTER_RETURN_VAL(cont.get_register(reg)));
            }

            COMMIT_MEMORY    =>     {
               println!("Get Registers");
               cont.commit_memory();
               self.transport.send(Message::COMMIT_MEMORY_SUCCESS);
            }
            
            GET_STATE    =>     {
               println!("Get Registers");
               let state = cont.get_state();
               self.transport.send(Message::GET_STATE_RETURN_VAL(state));
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
