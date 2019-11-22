use lc3_isa::{Addr, Bits, Instruction, Reg, Word};
use lc3_traits::control::{Control, Event, State};
use lc3_traits::memory::{Memory, MemoryMiscError};
use lc3_traits::peripherals::input::Input;
use lc3_traits::peripherals::{PeripheralSet, Peripherals};

use core::convert::TryInto;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use crate::interp::{InstructionInterpreter, InstructionInterpreterPeripheralAccess};
use core::cell::Cell;

use core::future::Future;
use core::ops::Deref;
use lc3_traits::control::State::Paused;
use lc3_traits::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHES};
use lc3_traits::error::Error;
use lc3_traits::peripherals::adc::{Adc, AdcPinArr, AdcReadError, AdcState};
use lc3_traits::peripherals::clock::Clock;
use lc3_traits::peripherals::gpio::{Gpio, GpioPinArr, GpioReadError, GpioState};
use lc3_traits::peripherals::pwm::{Pwm, PwmPinArr, PwmState};
use lc3_traits::peripherals::timers::{TimerArr, TimerState, Timers};
use lc3_traits::peripherals::*;
use std::f32::MAX;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Simulator<'a, I: InstructionInterpreter + InstructionInterpreterPeripheralAccess<'a>>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    interp: I,
    breakpoints: [Option<Addr>; MAX_BREAKPOINTS],
    watchpoints: [Option<(Addr, Word)>; MAX_MEMORY_WATCHES], // TODO: change to throw these when the location being watched to written to; not just when the value is changed...
    num_set_breakpoints: usize,
    num_set_watchpoints: usize,
    state: State,
    _i: PhantomData<&'a ()>,
}

impl<'a, I: InstructionInterpreterPeripheralAccess<'a> + Default> Default for Simulator<'a, I>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<'a, I: InstructionInterpreterPeripheralAccess<'a>> Simulator<'a, I>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    fn new(interp: I) -> Self {
        Self {
            interp,
            breakpoints: [None; MAX_BREAKPOINTS],
            watchpoints: [None; MAX_MEMORY_WATCHES],
            num_set_breakpoints: 0,
            num_set_watchpoints: 0,
            state: State::Paused,
            _i: PhantomData,
        }
    }
}

impl<'a, I: InstructionInterpreterPeripheralAccess<'a>> Control for Simulator<'a, I>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    type EventFuture = SimFuture;

    fn get_pc(&self) -> Addr {
        self.interp.get_pc()
    }

    fn set_pc(&mut self, addr: Addr) {
        self.interp.set_pc(addr)
    }

    fn get_register(&self, reg: Reg) -> Word {
        self.interp.get_register(reg)
    }

    fn set_register(&mut self, reg: Reg, data: Word) {
        self.interp.set_register(reg, data)
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.interp.set_word_unchecked(addr, word)
    }

    fn read_word(&self, addr: Addr) -> Word {
        self.interp.get_word_unchecked(addr)
    }

    fn commit_memory(&mut self) -> Result<(), MemoryMiscError> {
        self.interp.commit_memory()
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        if self.num_set_breakpoints < MAX_BREAKPOINTS {
            self.breakpoints[self.num_set_breakpoints] = Option::from(addr);
            self.num_set_breakpoints += 1;
            Ok(self.num_set_breakpoints)
        } else {
            Err(())
        }
    }

    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        if idx < self.num_set_breakpoints {
            self.breakpoints[idx] = None;
            self.num_set_breakpoints -= 1;

            let mut i = idx;
            while self.breakpoints[i + 1] != None {
                self.breakpoints[i] = self.breakpoints[i + 1];
                i += 1;
            }

            Ok(())
        } else {
            Err(())
        }
    }

    //     fn get_breakpoint(&self) ->

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        self.breakpoints
    }

    // TODO: breakpoints and watchpoints look macroable
    fn set_memory_watch(&mut self, addr: Addr, data: Word) -> Result<usize, ()> {
        if self.num_set_watchpoints < MAX_MEMORY_WATCHES {
            self.watchpoints[self.num_set_watchpoints] = Option::from((addr, data));
            self.num_set_watchpoints += 1;
            Ok(self.num_set_watchpoints)
        } else {
            Err(())
        }
    }

    fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()> {
        if idx < self.num_set_watchpoints {
            self.watchpoints[idx] = None;
            self.num_set_watchpoints -= 1;

            let mut i = idx;
            while self.watchpoints[i + 1] != None {
                self.watchpoints[i] = self.watchpoints[i + 1];
                i += 1;
            }

            Ok(())
        } else {
            Err(())
        }
    }

    fn get_memory_watches(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHES] {
        self.watchpoints
    }

    fn run_until_event(&mut self) -> Self::EventFuture {
        // DO NOT IMPLEMENT, yet
        unimplemented!()
    }

    fn step(&mut self) {
        match self.interp.step() {
            Running => {
                self.state = State::Paused;
            }
            Halted => {
                self.state = State::Halted;
            }
        }
    }

    fn pause(&mut self) {
        unimplemented!()
    }

    fn get_state(&self) -> State {
        self.state
    }

    fn get_error(&self) -> Option<Error> {
        unimplemented!()
    }

    fn get_gpio_states(&self) -> GpioPinArr<GpioState> {
        Gpio::get_states(self.interp.get_peripherals())
    }

    fn get_gpio_reading(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        Gpio::read_all(self.interp.get_peripherals())
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState> {
        Adc::get_states(self.interp.get_peripherals())
    }

    fn get_adc_reading(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
        Adc::read_all(self.interp.get_peripherals())
    }

    fn get_timer_states(&self) -> TimerArr<TimerState> {
        Timers::get_states(self.interp.get_peripherals())
    }

    fn get_timer_config(&self) -> TimerArr<Word> {
        Timers::get_periods(self.interp.get_peripherals())
    }

    fn get_pwm_states(&self) -> PwmPinArr<PwmState> {
        Pwm::get_states(self.interp.get_peripherals())
    }

    fn get_pwm_config(&self) -> PwmPinArr<u8> {
        Pwm::get_duty_cycles(self.interp.get_peripherals())
    }

    fn get_clock(&self) -> Word {
        Clock::get_milliseconds(self.interp.get_peripherals())
    }
}

pub struct SimFuture;

impl Future for SimFuture {
    type Output = Event;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::isa::Instruction;

//     // Test that the instructions work
//     // Test that the unimplemented instructions do <something>

//     fn interp_test_runner<'a, M: Memory, P: Peripherals<'a>>(
//         insns: Vec<Instruction>,
//         num_steps: Option<usize>,
//         regs: [Option<Word>; 8],
//         pc: Addr,
//         memory_locations: Vec<(Addr, Word)>,
//     ) {
//         let mut interp = Simulator::<M, P>::default();

//         let mut addr = 0x3000;
//         for insn in insns {
//             interp.set_word(addr, insn.into());
//             addr += 2;
//         }

//         if let Some(num_steps) = num_steps {
//             for _ in 0..num_steps {
//                 interp.step();
//             }
//         } else {
//             // TODO! (run until halted)
//         }

//         for (idx, r) in regs.iter().enumerate() {
//             if let Some(reg_word) = r {
//                 assert_eq!(interp.get_register(idx.into()), *reg_word);
//             }
//         }
//     }

//     #[test]
//     //NO-OP do nothing run a cycle test
//     fn nop() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Br {
//                 n: true,
//                 z: true,
//                 p: true,
//                 offset11: -1,
//             }],
//             Same(1),
//             [None, None, None, None, None, None, None, None],
//             0x3000,
//             vec![],
//         )
//     }
//     //0+1=1 Basic Add
//     #[test]
//     fn add_reg_test() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 1,
//                 },
//                 AddReg {
//                     dr: 2,
//                     sr1: 1,
//                     sr2: 0,
//                 },
//             ],
//             Some(1),
//             [Some(0), Some(1), Some(1), None, None, None, None, None],
//             0x3001,
//             vec![],
//         )
//     }
//     //AddImm Test with R0(0) + !
//     #[test]
//     fn AddImmTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::AddImm {
//                 dr: R0,
//                 sr1: R0,
//                 imm5: 1,
//             }],
//             Some(1),
//             [1, None, None, None, None, None, None, None],
//             0x3001,
//             vec![],
//         )
//     }
//     //AndReg Test with R0(1) and R1(2) to R0(expected 3)
//     #[test]
//     fn AndRegTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 AddImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 2,
//                 },
//                 AndReg {
//                     dr: R0,
//                     sr1: R0,
//                     sr2: R1,
//                 },
//             ],
//             Some(3),
//             [3, 2, None, None, None, None, None, None],
//             0x3003,
//             vec![],
//         )
//     }
//     //AndImm Test with R1 (1) and 0
//     #[test]
//     fn AndImmTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 1,
//                 },
//                 AndImm {
//                     dr: R1,
//                     sr1: R1,
//                     imm5: 0,
//                 },
//             ],
//             Some(2),
//             [0, None, None, None, None, None, None, None],
//             0x3002,
//             vec![],
//         )
//     }
//     //ST Test which stores 1 into x3001
//     #[test]
//     fn StTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 St { sr: R0, offset9: 0 },
//             ],
//             Some(2),
//             [1, None, None, None, None, None, None, None],
//             0x3002,
//             vec![(0x3001, 1)],
//         )
//     }
//     //LD Test with R0 and memory
//     #[test]
//     fn LdTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 St { sr: R0, offset9: 1 },
//                 Ld { dr: R0, offset9: 0 },
//             ],
//             Some(3),
//             [3001, None, None, None, None, None, None, None],
//             0x3003,
//             vec![(0x3001, 1)],
//         )
//     }
//     //LDR Test with R0 and memory
//     #[test]
//     fn LdrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 St { sr: R0, offset9: 0 },
//                 Ldr {
//                     dr: R1,
//                     offset9: -1,
//                 },
//             ],
//             Some(3),
//             [1, 3001, None, None, None, None, None, None],
//             0x3003,
//             vec![(0x3001, 1)],
//         )
//     }
//     //Load x3000 into R1
//     #[test]
//     fn LeaTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R0, offset9: 0 }],
//             Some(1),
//             [3000, None, None, None, None, None, None, None],
//             0x3001,
//             vec![],
//         )
//     }
//     // STR test with offset store into lea using 3000
//     #[test]
//     fn StrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R1, offset9: 0 },
//                 Lea { dr: R2, offset9: 1 },
//                 Str {
//                     sr: R2,
//                     base: R1,
//                     offset6: 1,
//                 },
//             ],
//             Some(3),
//             [None, None, None, None, None, None, None, None],
//             0x3003,
//             vec![(x3004, 3000)],
//         )
//     }
//     //not test
//     #[test]
//     fn NotTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::AddImm {
//                     dr: R0,
//                     sr1: R0,
//                     imm5: 1,
//                 },
//                 Not { dr: R1, sr: R0 },
//             ],
//             Some(2),
//             [1, 0, None, None, None, None, None, None],
//             0x3002,
//             vec![],
//         )
//     }
//     //ldi Test using location 3000 and loading value of memory into register, using 3002 and 3001 holding 3000 as reference
//     #[test]
//     fn LdiTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R0, offset9: 0 },
//                 St { sr: R0, offset9: 0 },
//                 St {
//                     sr: R0,
//                     offset9: -2,
//                 },
//                 Ldi {
//                     dr: R2,
//                     offset9: -1,
//                 },
//             ],
//             Some(4),
//             [1, None, 3000, None, None, None, None, None],
//             0x3004,
//             vec![(x3001, 3000), (x3000, 3000)],
//         )
//     }
//     //jumps to R7 register, loaded with memory address 3005
//     #[test]
//     fn RetTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R7, offset9: 5 }, Ret],
//             Some(2),
//             [None, None, None, None, None, None, None, 3005],
//             0x3005,
//             vec![],
//         )
//     }
//     //STI test, stores 3000 in register 1 and sets that to the memory at x3002 so sti writes to memory location 3000
//     #[test]
//     fn StiTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R0, offset9: 0 },
//                 St { sr: R0, offset6: 2 },
//                 AddImm {
//                     dr: R3,
//                     sr1: R3,
//                     imm5: 1,
//                 },
//                 Sti { sr: R3, offset9: 0 },
//             ],
//             Some(4),
//             [3000, None, None, 1, None, None, None, None],
//             0x3004,
//             vec![(x3003, 3000), (x3000, 1)],
//         )
//     }
//     //Jump Test, switch PC to value in register
//     #[test]
//     fn JmpTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R0, offset9: 0 }, Jmp { base: R0 }],
//             Some(2),
//             [3000, None, None, None, None, None, None, None],
//             0x3000,
//             vec![],
//         )
//     }
//     //jsrr test, jumps to location 3005 and stores 3001 in r7
//     #[test]
//     fn JsrrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![Instruction::Lea { dr: R0, offset9: 5 }, Jsrr { base: R0 }],
//             Some(2),
//             [3000, None, None, None, None, None, None, 3001],
//             0x3005,
//             vec![],
//         )
//     }
//     //jsr test, jumps back to queue location from r7
//     #[test]
//     fn JsrTest() {
//         interp_test_runner::<MemoryShim, _>(
//             vec![
//                 Instruction::Lea { dr: R0, offset9: 5 },
//                 St { sr: R0, offset6: 2 },
//                 Jsr { offset11: 1 },
//             ],
//             Some(3),
//             [3000, None, None, None, None, None, None, 3001],
//             0x3000,
//             vec![],
//         )
//     }
// }
