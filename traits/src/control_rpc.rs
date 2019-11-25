// extern crate futures; // 0.1.23
// extern crate rand;

use crate::control::{Control, Event, State, MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use crate::error::Error;
use lc3_isa::Reg;
use crate::peripherals::{gpio::*, timers::*, pwm::*, adc::*};
use crate::memory::MemoryMiscError;

use lc3_isa::*;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use serde::{Deserialize, Serialize};
//use std::sync::mpsc::{Receiver, Sender};

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

    GET_BREAKPOINTS_RETURN_VAL([Option<(Addr)>; MAX_BREAKPOINTS]),
    GET_MAX_BREAKPOINTS,
    SET_MEMORY_WATCH(u16, u16),
    SET_MEMORY_WATCH_SUCCESS(Result<usize,()>),
    UNSET_MEMORY_WATCH(usize),
    UNSET_MEMORY_WATCH_SUCCESS(Result<(),()>),
    GET_MEMORY_WATCHES,
    GET_MEMORY_WATCHES_RETURN_VAL([Option<(Addr,Word)>; MAX_MEMORY_WATCHES]),
    GET_MAX_MEMORY_WATCHES,
    STEP,
    STEP_RETURN_STATE(State),
    RUN_UNTIL_EVENT,
    ISSUED_RUN_UNTIL_EVENT,
    SET_REGISTER(Reg, Word),
    SET_REGISTER_SUCCESS,
    GET_REGISTER(Reg),
    GET_REGISTER_RETURN_VAL(u16),
    COMMIT_MEMORY,
    COMMIT_MEMORY_SUCCESS(Result<(), MemoryMiscError>),
    GET_STATE,
    GET_STATE_RETURN_VAL(State),
    GET_GPIO_STATES,
    GET_GPIO_STATES_RETURN_VAL(GpioPinArr<GpioState>),
    GET_GPIO_READING,
    GET_GPIO_READING_RETURN_VAL(GpioPinArr<Result<bool, GpioReadError>>),
    GET_ADC_STATES,
    GET_ADC_STATES_RETURN_VAL(AdcPinArr<AdcState>),
    GET_ADC_READING,
    GET_ADC_READING_RETURN_VAL(AdcPinArr<Result<u8, AdcReadError>>),
    GET_TIMER_STATES,
    GET_TIMER_STATES_RETURN_VAL(TimerArr<TimerState>),
    GET_TIMER_CONFIG,
    GET_TIMER_CONFIG_RETURN_VAL(TimerArr<Word>),
    GET_PWM_STATES,
    GET_PWM_STATES_RETURN_VAL(PwmPinArr<PwmState>),
    GET_PWM_CONFIG,
    GET_PWM_CONFIG_RETURN_VAL(PwmPinArr<u8>),
    GET_CLOCK,
    GET_CLOCK_RETURN_VAL(Word),


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

    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        let mut res: Result<(), MemoryMiscError>;// = Err(());
        self.transport.send(Message::COMMIT_MEMORY);
        if let Some(m) = self.transport.get() {
            if let Message::COMMIT_MEMORY_SUCCESS(status) = m {
                res = status;
            } else {
                panic!();
            }

        }
        else{
            panic!();
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
        let mut ret: [Option<(Addr)>; MAX_BREAKPOINTS] = [None; MAX_BREAKPOINTS];
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

    fn set_memory_watch(&mut self, addr: Addr, data: Word) -> Result<usize, ()> {
        self.transport.send(Message::SET_MEMORY_WATCH(addr, data));
        let mut res: Result<usize, ()> = Err(());
        // self.transport.send(Message::SET_MEMORY_WATCH(addr));
        if let Some(m) = self.transport.get() {
            if let Message::SET_MEMORY_WATCH_SUCCESS(ret) = m {
                res = ret;
            } else {
                //res = Err(());
                panic!();
            }
        }
        res
    }

    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()> {
        self.transport.send(Message::UNSET_MEMORY_WATCH(idx));
        let mut res: Result<(), ()> = Err(());
        // self.transport.send(Message::SET_MEMORY_WATCH(addr));
        if let Some(m) = self.transport.get() {
            if let Message::UNSET_MEMORY_WATCH_SUCCESS(ret) = m {
                res = ret;
            } else {
                panic!();
            }
        }
        res
    }

    fn get_memory_watches(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] {
        let mut ret: [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] = [None; MAX_MEMORY_WATCHES];
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
    fn step(&mut self) -> State {
        let mut ret: State = State::Paused;
        self.transport.send(Message::STEP);
        if let Some(m) = self.transport.get() {
            if let Message::STEP_RETURN_STATE(state) = m {
                ret=state;
            } else {
                panic!();
            }
        }
        ret
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
    fn get_gpio_states(&self) -> GpioPinArr<GpioState>{
        let mut ret: GpioPinArr<GpioState>;// = State::Paused;
        self.transport.send(Message::GET_GPIO_STATES);

        if let Some(m) = self.transport.get() {
            if let Message::GET_GPIO_STATES_RETURN_VAL(states) = m {
                ret = states;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }

    fn get_gpio_reading(&self) -> GpioPinArr<Result<bool, GpioReadError>>{
        let mut ret: GpioPinArr<Result<bool, GpioReadError>>;// = State::Paused;
        self.transport.send(Message::GET_GPIO_READING);

        if let Some(m) = self.transport.get() {
            if let Message::GET_GPIO_READING_RETURN_VAL(gpio) = m {
                ret = gpio;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState>{
        let mut ret: AdcPinArr<AdcState>;
        self.transport.send(Message::GET_ADC_STATES);

        if let Some(m) = self.transport.get() {
            if let Message::GET_ADC_STATES_RETURN_VAL(adc) = m {
                ret = adc;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret


    }
    fn get_adc_reading(&self) -> AdcPinArr<Result<u8, AdcReadError>>{
         let mut ret: AdcPinArr<Result<u8, AdcReadError>>;// = State::Paused;
        self.transport.send(Message::GET_ADC_STATES);

        if let Some(m) = self.transport.get() {
            if let Message::GET_ADC_READING_RETURN_VAL(addr) = m {
                ret = addr;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret

    }
    fn get_timer_states(&self) -> TimerArr<TimerState>{
        let mut ret: TimerArr<TimerState>;// = State::Paused;
        self.transport.send(Message::GET_TIMER_STATES);

        if let Some(m) = self.transport.get() {
            if let Message::GET_TIMER_STATES_RETURN_VAL(addr) = m {
                ret = addr;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret

    }
    fn get_timer_config(&self) -> TimerArr<Word>{
        let mut ret: TimerArr<Word>;// = State::Paused;
        self.transport.send(Message::GET_TIMER_CONFIG);

        if let Some(m) = self.transport.get() {
            if let Message::GET_TIMER_CONFIG_RETURN_VAL(addr) = m {
                ret = addr;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret

    }
    fn get_pwm_states(&self) -> PwmPinArr<PwmState>{
        let mut ret: PwmPinArr<PwmState>;
        self.transport.send(Message::GET_PWM_STATES);

        if let Some(m) = self.transport.get() {
            if let Message::GET_PWM_STATES_RETURN_VAL(states) = m {
                ret = states;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret

    }
    fn get_pwm_config(&self) -> PwmPinArr<u8>{
        let mut ret: PwmPinArr<u8>;
        self.transport.send(Message::GET_PWM_CONFIG);

        if let Some(m) = self.transport.get() {
            if let Message::GET_PWM_CONFIG_RETURN_VAL(conf) = m {
                ret = conf;
            } else {
                panic!();
            }
        } else {
            panic!();
        }
        ret
    }
    fn get_clock(&self) -> Word{
        16

    }
    fn reset(&mut self){

    }
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

                SET_MEMORY_WATCH(addr, word) => {
                    let res =  cont.set_memory_watch(addr, word);
                    self.transport.send(SET_MEMORY_WATCH_SUCCESS(res));
                }

                UNSET_MEMORY_WATCH(idx) => {
                    let res= cont.unset_memory_watch(idx);
                    self.transport.send(UNSET_MEMORY_WATCH_SUCCESS(res));
                }

                GET_MAX_BREAKPOINTS => (),    // TODO: do
                GET_MAX_MEMORY_WATCHES => (), // TODO: do

                STEP => {
                    let state=cont.step();
                    self.transport.send(STEP_RETURN_STATE(state));
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
                    let res = cont.commit_memory();
                    self.transport.send(COMMIT_MEMORY_SUCCESS(res));
                }

                GET_STATE => {
                    let state = cont.get_state();
                    self.transport.send(GET_STATE_RETURN_VAL(state));
                }

                GET_GPIO_STATES =>{
                    let state = cont.get_gpio_states();
                    self.transport.send(GET_GPIO_STATES_RETURN_VAL(state));
                }
                GET_GPIO_READING =>{
                    let state = cont.get_gpio_reading();
                    self.transport.send(GET_GPIO_READING_RETURN_VAL(state));               
                }
                GET_ADC_READING =>{
                    let state = cont.get_adc_reading();
                    self.transport.send(GET_ADC_READING_RETURN_VAL(state));
                }
                GET_ADC_STATES => {
                    let state = cont.get_adc_states();
                    self.transport.send(GET_ADC_STATES_RETURN_VAL(state));
                }
                GET_TIMER_STATES => {
                    let state = cont.get_timer_states();
                    self.transport.send(GET_TIMER_STATES_RETURN_VAL(state));
                }

                GET_TIMER_CONFIG =>{
                    let state = cont.get_timer_config();
                    self.transport.send(GET_TIMER_CONFIG_RETURN_VAL(state));
                }

                GET_PWM_STATES => {
                    let state = cont.get_pwm_states();
                    self.transport.send(GET_PWM_STATES_RETURN_VAL(state));
                }

                GET_PWM_CONFIG =>{
                    let state = cont.get_pwm_config();
                    self.transport.send(GET_PWM_CONFIG_RETURN_VAL(state));
                }
                GET_CLOCK =>{
                      let state = cont.get_clock();
                    self.transport.send(GET_CLOCK_RETURN_VAL(state));                  
                }

                _ => unreachable!(),
            }
        }

        num_executed_messages
    }
}

// pub struct MpscTransport {
//     tx: Sender<std::string::String>,
//     rx: Receiver<std::string::String>,
// }

// impl TransportLayer for MpscTransport {
//     fn send(&self, message: Message) -> Result<(), ()> {
//         let point = message;
//         let serialized = serde_json::to_string(&point).unwrap();

//         self.tx.send(serialized).unwrap();

//         Ok(())
//     }

//     fn get(&self) -> Option<Message> {
//         let deserialized: Message = serde_json::from_str(&self.rx.recv().unwrap()).unwrap();

//         println!("deserialized = {:?}", deserialized);
//         Some(deserialized)
//     }
// }

// pub fn mpsc_transport_pair() -> (MpscTransport, MpscTransport) {
//     let (tx_h, rx_h) = std::sync::mpsc::channel();
//     let (tx_d, rx_d) = std::sync::mpsc::channel();

//     let host_channel = MpscTransport { tx: tx_h, rx: rx_d };
//     let device_channel = MpscTransport { tx: tx_d, rx: rx_h };

//     (host_channel, device_channel)
// }

// //fn run_channel()
