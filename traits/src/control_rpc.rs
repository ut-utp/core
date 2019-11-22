// extern crate futures; // 0.1.23
// extern crate rand;

use crate::control::{Control, Event, Reg, State, MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use crate::error::Error;

use lc3_isa::*;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use serde::{Deserialize, Serialize};

static mut CURRENT_STATE: State = State::Paused; // TODO: what to do?

pub struct CurrentEvent {
    CurrentEvent: Event,
    CurrentState: State,
}

impl Future for CurrentEvent {
    type Output = Event;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match CURRENT_STATE {
                Paused => {
                    CURRENT_STATE = State::Paused;
                    Poll::Ready(Event::Interrupted)
                }
                _ => {
                    cx.waker().clone().wake();
                    Poll::Pending
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    GET_PC,
    GET_PC_RETURN_VAL(Addr),
    SET_PC(u16),
    SET_PC_SUCCESS,
    WRITE_WORD(u16, u16),
    WRITE_WORD_SUCCESS,
    READ_WORD(u16),
    READ_WORD_RETURN_VAL(u16),
    PAUSE,
    PAUSE_SUCCESS,
    SET_BREAKPOINT(u16),
    SET_BREAKPOINT_SUCCESS,
    UNSET_BREAKPOINT(usize),
    UNSET_BREAKPOINT_SUCCESS,
    GET_BREAKPOINTS,
    GET_BREAKPOINTS_RETURN_VAL([Option<Addr>; MAX_BREAKPOINTS]),
    GET_MAX_BREAKPOINTS,
    SET_MEMORY_WATCH(u16),
    SET_MEMORY_WATCH_SUCCESS,
    UNSET_MEMORY_WATCH(usize),
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
        let mut ret: u16 = 0;
        self.transport.send(Message::GET_PC);

        if let Some(m) = self.transport.get() {
            if let Message::GET_PC_RETURN_VAL(addr) = m {
                ret = addr;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
        //panic!();
    }

    fn get_register(&self, reg: Reg) -> Word {
        let mut ret: u16 = 0;
        self.transport.send(Message::GET_REGISTER(reg));

        if let Some(m) = self.transport.get() {
            if let Message::GET_REGISTER_RETURN_VAL(val) = m {
                ret = val;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }

    fn set_register(&mut self, reg: Reg, data: Word) {
        self.transport.send(Message::SET_REGISTER(reg, data));
        if let Some(m) = self.transport.get() {
            if let Message::SET_REGISTER_SUCCESS = m {
            } else {
                panic!();
            }
        }
    } // Should be infallible.

    // fn get_registers_and_pc(&self) -> ([Word; 9], Word); // TODO

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.transport.send(Message::WRITE_WORD(addr, word));
        if let Some(m) = self.transport.get() {
            if let Message::WRITE_WORD_SUCCESS = m {
            } else {
                panic!();
            }
        }
    }

    fn read_word(&self, addr: Addr) -> Word {
        let mut ret: u16 = 0;
        self.transport.send(Message::READ_WORD(addr));
        if let Some(m) = self.transport.get() {
            if let Message::READ_WORD_RETURN_VAL(word) = m {
                ret = word;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }

    fn commit_memory(&self) -> Result<(), ()> {
        let mut res: Result<(), ()> = Err(());
        self.transport.send(Message::COMMIT_MEMORY);
        if let Some(m) = self.transport.get() {
            if let Message::COMMIT_MEMORY_SUCCESS = m {
                res = Ok(());
            } else {
                res = Err(());
            }
        }
        res
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        let mut res: Result<usize, ()> = Err(());
        self.transport.send(Message::SET_BREAKPOINT(addr));
        if let Some(m) = self.transport.get() {
            if let Message::SET_BREAKPOINT_SUCCESS = m {
                res = Ok(1)
            } else {
                res = Err(());
            }
        }
        res
    }

    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        self.transport.send(Message::UNSET_BREAKPOINT(idx));
        //self.transport.send(Message::SET_MEMORY_WATCH(addr));
        let mut res: Result<(), ()> = Err(());
        // self.transport.send(Message::SET_MEMORY_WATCH(addr));
        if let Some(m) = self.transport.get() {
            if let Message::UNSET_BREAKPOINT_SUCCESS = m {
                res = Ok(());
            } else {
                res = Err(());
            }
        }
        res
    }

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        let mut ret: [Option<Addr>; MAX_BREAKPOINTS] = [None; MAX_BREAKPOINTS];
        self.transport.send(Message::GET_BREAKPOINTS);

        if let Some(m) = self.transport.get() {
            if let Message::GET_BREAKPOINTS_RETURN_VAL(val) = m {
                ret = val;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }

    fn get_max_breakpoints() -> usize {
        // TODO: Actually proxy this
        MAX_BREAKPOINTS
    }

    fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()> {
        self.transport.send(Message::SET_MEMORY_WATCH(addr));
        let mut res: Result<usize, ()> = Err(());
        // self.transport.send(Message::SET_MEMORY_WATCH(addr));
        if let Some(m) = self.transport.get() {
            if let Message::SET_MEMORY_WATCH_SUCCESS = m {
                res = Ok(1);
            } else {
                res = Err(());
            }
        }
        res
    }

    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()> {
        self.transport.send(Message::UNSET_MEMORY_WATCH(idx));
        let mut res: Result<(), ()> = Err(());
        // self.transport.send(Message::SET_MEMORY_WATCH(addr));
        if let Some(m) = self.transport.get() {
            if let Message::UNSET_MEMORY_WATCH_SUCCESS = m {
                res = Ok(());
            } else {
                res = Err(());
            }
        }
        res
    }

    fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES] {
        let mut ret: [Option<Addr>; MAX_MEMORY_WATCHES] = [None; MAX_MEMORY_WATCHES];
        self.transport.send(Message::GET_MEMORY_WATCHES);

        if let Some(m) = self.transport.get() {
            if let Message::GET_MEMORY_WATCHES_RETURN_VAL(val) = m {
                ret = val;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }

    fn get_max_memory_watches() -> usize {
        MAX_MEMORY_WATCHES
    }

    // Execution control functions:
    fn run_until_event(&mut self) -> Self::EventFuture {
        self.transport.send(Message::RUN_UNTIL_EVENT);
        if let Some(m) = self.transport.get() {
            if let Message::ISSUED_RUN_UNTIL_EVENT = m {
            } else {
                panic!();
            }
        }
        unsafe {
            CURRENT_STATE = State::RunningUntilEvent;
        }
        CurrentEvent {
            CurrentEvent: Event::Interrupted,
            CurrentState: State::RunningUntilEvent,
        }
    } // Can be interrupted by step or pause.
    fn step(&mut self) {
        self.transport.send(Message::STEP);
        if let Some(m) = self.transport.get() {
            if let Message::STEP_SUCCESSFUL = m {
            } else {
                panic!();
            }
        }
    }
    fn pause(&mut self) {
        self.transport.send(Message::PAUSE);
        if let Some(m) = self.transport.get() {
            if let Message::PAUSE_SUCCESS = m {
            } else {
                panic!();
            }
        }
    }

    fn get_state(&self) -> State {
        let mut ret: State = State::Paused;
        self.transport.send(Message::GET_STATE);

        if let Some(m) = self.transport.get() {
            if let Message::GET_STATE_RETURN_VAL(addr) = m {
                ret = addr;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
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
        self.transport.send(Message::SET_PC(addr));
        if let Some(m) = self.transport.get() {
            if let Message::SET_PC_SUCCESS = m {
            } else {
                panic!();
            }
        }
    } // Should be infallible.
}

pub struct Client<T: TransportLayer> {
    pub transport: T,
}

impl<T: TransportLayer> Client<T> {
    // Check for messages and execute them on something that implements the control
    // interface.

    pub fn step<C: Control>(&mut self, cont: &mut C) -> usize {
        let mut num_executed_messages = 0;

        while let Some(m) = self.transport.get() {
            use Message::*;

            num_executed_messages += 1;

            match m {
                GET_PC => {
                    self.transport.send(GET_PC_RETURN_VAL(cont.get_pc()));
                }

                GET_PC_RETURN_VAL(h) => {}

                SET_PC(val) => {
                    cont.set_pc(val);
                    self.transport.send(SET_PC_SUCCESS);
                }

                SET_REGISTER(reg, word) => {
                    cont.set_register(reg, word);
                    self.transport.send(SET_REGISTER_SUCCESS);
                }

                RUN_UNTIL_EVENT => {
                    cont.run_until_event();
                    self.transport.send(ISSUED_RUN_UNTIL_EVENT);
                }

                WRITE_WORD(word, value) => {
                    cont.write_word(word, value);
                    self.transport.send(WRITE_WORD_SUCCESS);
                }

                PAUSE => {
                    cont.pause();
                    self.transport.send(PAUSE_SUCCESS);
                }
                SET_BREAKPOINT(addr) => {
                    cont.set_breakpoint(addr);
                    self.transport.send(SET_BREAKPOINT_SUCCESS);
                }
                UNSET_BREAKPOINT(addr) => {
                    cont.unset_breakpoint(addr);
                    self.transport.send(UNSET_BREAKPOINT_SUCCESS);
                }

                GET_BREAKPOINTS => {
                    let breaks = cont.get_breakpoints();
                    self.transport.send(GET_BREAKPOINTS_RETURN_VAL(breaks));
                }

                SET_MEMORY_WATCH(addr) => {
                    cont.set_memory_watch(16);
                    self.transport.send(SET_MEMORY_WATCH_SUCCESS);
                }

                UNSET_MEMORY_WATCH(idx) => {
                    cont.unset_memory_watch(idx);
                    self.transport.send(UNSET_MEMORY_WATCH_SUCCESS);
                }

                GET_MAX_BREAKPOINTS => (),    // TODO: do
                GET_MAX_MEMORY_WATCHES => (), // TODO: do

                STEP => {
                    cont.step();
                    self.transport.send(STEP_SUCCESSFUL);
                }

                READ_WORD(addr) => {
                    self.transport
                        .send(READ_WORD_RETURN_VAL(cont.read_word(addr)));
                }
                GET_MEMORY_WATCHES => {
                    cont.get_memory_watches();
                }

                GET_REGISTER(reg) => {
                    self.transport
                        .send(GET_REGISTER_RETURN_VAL(cont.get_register(reg)));
                }

                COMMIT_MEMORY => {
                    cont.commit_memory();
                    self.transport.send(COMMIT_MEMORY_SUCCESS);
                }

                GET_STATE => {
                    let state = cont.get_state();
                    self.transport.send(GET_STATE_RETURN_VAL(state));
                }

                _ => unreachable!(),
            }
        }

        num_executed_messages
    }
}
