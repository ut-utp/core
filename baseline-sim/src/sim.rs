//! TODO!

use crate::interp::{InstructionInterpreter, InstructionInterpreterPeripheralAccess, MachineState};
use crate::mem_mapped::{MemMapped, KBDR};

use lc3_isa::{Addr, Reg, Word};
use lc3_traits::control::{Control, Event, State};
use lc3_traits::control::control::{MAX_BREAKPOINTS, MAX_MEMORY_WATCHPOINTS};
use lc3_traits::control::metadata::{Identifier, ProgramMetadata, DeviceInfo};
use lc3_traits::control::load::{
    PageIndex, PageWriteStart, StartPageWriteError, PageChunkError,
    FinishPageWriteError, LoadApiSession, Offset, CHUNK_SIZE_IN_WORDS,
    PAGE_SIZE_IN_WORDS, Index, hash_page, PageAccess,
};
use lc3_traits::control::rpc::{
    EventFutureSharedStatePorcelain, SimpleEventFutureSharedState, EventFuture
};
use lc3_traits::error::Error;
use lc3_traits::peripherals::adc::{Adc, AdcPinArr, AdcReadError, AdcState};
use lc3_traits::peripherals::clock::Clock;
use lc3_traits::peripherals::gpio::{Gpio, GpioPinArr, GpioReadError, GpioState};
use lc3_traits::peripherals::pwm::{Pwm, PwmPinArr, PwmState};
use lc3_traits::peripherals::timers::{TimerArr, TimerState, Timers};
use lc3_traits::peripherals::Peripherals;

// use core::future::Future;
use core::marker::PhantomData;
use core::ops::Deref;
use core::fmt::{self, Debug};
// use core::pin::Pin;
// use core::task::{Context, Poll};

#[derive(Clone)]
enum LoadApiState {
    NoSession,
    Session {
        checksum: u64,
        page_idx: PageIndex,
        page: [Word; PAGE_SIZE_IN_WORDS as usize],
    }
}

impl Default for LoadApiState {
    fn default() -> Self { Self::NoSession }
}

impl Debug for LoadApiState {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadApiState::NoSession => write!(fmt, "LoadApiState::NoSession"),
            LoadApiState::Session { checksum, page_idx, .. } => {
                write!(fmt,
                    "LoadApiState::Session {{ checksum: {:?}, page_idx: {:?}, page: [ ... ] }}",
                    checksum,
                    page_idx
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Simulator<'int, 'ss, I: InstructionInterpreter + InstructionInterpreterPeripheralAccess<'int>, S: EventFutureSharedStatePorcelain = SimpleEventFutureSharedState>
where
    <I as Deref>::Target: Peripherals<'int>,
{
    interp: I,
    breakpoints: [Option<Addr>; MAX_BREAKPOINTS],
    watchpoints: [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS], // TODO: change to throw these when the location being watched to written to; not just when the value is changed...
    num_set_breakpoints: usize,
    num_set_watchpoints: usize,
    state: State,
    shared_state: Option<&'ss S>,
    load_api_state: LoadApiState,
    _i: PhantomData<&'int ()>,
}

impl<'a, 's, I: InstructionInterpreterPeripheralAccess<'a> + Default, S: EventFutureSharedStatePorcelain> Default for Simulator<'a, 's, I, S>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<'a, 's, I: InstructionInterpreterPeripheralAccess<'a>, S: EventFutureSharedStatePorcelain> Simulator<'a, 's, I, S>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    // No longer public.
    fn new(interp: I) -> Self {
        Self {
            interp,
            breakpoints: [None; MAX_BREAKPOINTS],
            watchpoints: [None; MAX_MEMORY_WATCHPOINTS],
            num_set_breakpoints: 0,
            num_set_watchpoints: 0,
            state: State::Paused,
            shared_state: None,
            load_api_state: LoadApiState::default(),
            _i: PhantomData,
        }
    }

    pub fn new_with_state(interp: I, state: &'s S) -> Self {
        let mut sim = Self::new(interp);
        sim.set_shared_state(state);

        sim
    }

    pub fn set_shared_state(&mut self, state: &'s S) {
        self.shared_state = Some(state);
    }
}

// impl<'a, I: InstructionInterpreterPeripheralAccess<'a>> Simulator<'a, I>
// where
//     <I as Deref>::Target: Peripherals<'a>,
// {
//     pub fn get_interpreter(&mut self) -> &mut I {
//         &mut self.interp
//     }
// }

impl<'a, 's, I: InstructionInterpreterPeripheralAccess<'a>, S: EventFutureSharedStatePorcelain> Control for Simulator<'a, 's, I, S>
where
    <I as Deref>::Target: Peripherals<'a>,
{
    type EventFuture = EventFuture<'s, S>;

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

    fn read_word(&self, addr: Addr) -> Word {
        // Our one stateful read
        // TODO: banish this from the codebase
        if addr == <KBDR as MemMapped>::ADDR {
            return 0;
        }

        self.interp.get_word_unchecked(addr)
    }

    fn write_word(&mut self, addr: Addr, word: Word) {
        self.interp.set_word_unchecked(addr, word)
    }

    fn start_page_write(
        &mut self,
        page: LoadApiSession<PageWriteStart>,
        checksum: u64,
    ) -> Result<LoadApiSession<PageIndex>, StartPageWriteError> {
        use {LoadApiState::*, StartPageWriteError::*};
        match self.load_api_state {
            NoSession => {
                #[allow(unsafe_code)]
                let ret = unsafe { page.approve() }?;

                // If we're still here, set things up for the session:
                self.load_api_state = Session {
                    page_idx: ret.get(),
                    checksum,
                    page: [0; PAGE_SIZE_IN_WORDS as usize],
                };

                Ok(ret)
            },
            Session { page_idx, .. } => {
                Err(UnfinishedSessionExists { unfinished_page: page_idx })
            }
        }
    }

    fn send_page_chunk(
        &mut self,
        offset: LoadApiSession<Offset>,
        chunk: [Word; CHUNK_SIZE_IN_WORDS as usize],
    ) -> Result<(), PageChunkError> {
        use {LoadApiState::*, PageChunkError::*};
        match self.load_api_state {
            NoSession => Err(NoCurrentSession),
            Session { page_idx, ref mut page, .. } => {
                let start_addr = Index(page_idx).with_offset(offset.get().0);
                let end_addr = start_addr
                    .checked_add((CHUNK_SIZE_IN_WORDS - 1) as Addr)
                    .filter(|e| start_addr.page_idx() == e.page_idx())
                    .ok_or(ChunkCrossesPageBoundary { page: page_idx, received_address: start_addr })?;

                // let end_addr = if let Some(end_addr) = start_addr.checked_add((CHUNK_SIZE_IN_WORDS - 1) as Addr) {
                //     if start_addr.page_idx() != end_addr.page_idx()
                // } else {
                //     return Err(ChunkCrossesPageBoundary { page: page_idx, received_addr: start_addr })
                // }

                page[(start_addr.page_offset() as usize)..=(end_addr.page_offset() as usize)]
                    .copy_from_slice(&chunk);

                Ok(())
            }
        }

    }

    fn finish_page_write(
        &mut self,
        page_token: LoadApiSession<PageIndex>,
    ) -> Result<(), FinishPageWriteError> {
        use {LoadApiState::*, FinishPageWriteError::*};
        match self.load_api_state {
            NoSession => Err(NoCurrentSession),
            Session { page_idx, checksum, page } => {
                let res = (|| {
                    let page_idx_given = page_token.get();
                    if page_idx != page_idx_given {
                        return Err(SessionMismatch {
                            current_session_page: page_idx,
                            received_page: page_idx_given,
                        })
                    }

                    let computed_checksum = hash_page(&page);
                    if checksum != computed_checksum {
                        return Err(ChecksumMismatch {
                            page: page_idx,
                            given_checksum: checksum,
                            computed_checksum,
                        })
                    }

                    self.interp.commit_page(page_idx, &page);
                    Ok(())
                })();

                // No matter what (if we succeeded or we failed) this ends the
                // current session:
                self.load_api_state = NoSession;
                res
            }
        }
    }

    fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        // Scan for the next open slot:
        for (idx, bp) in self.breakpoints.iter_mut().enumerate() {
            if let Some(a) = bp {
                // For each set breakpoint, check if it's already the one we
                // want:
                // (note that this doesn't increment the number of set
                // breakpoints since it's not adding a new one)
                if addr == *a {
                    return Ok(idx)
                }
            } else {
                // If we reach an empty slot, use it.
                self.num_set_breakpoints += 1;
                *bp = Some(addr);
                return Ok(idx)
            }
        }

        Err(())
    }

    fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()> {
        if idx < MAX_BREAKPOINTS {
            self.breakpoints[idx].take().map(|_| {
                // If we actually removed a breakpoint, subtract the count:
                self.num_set_breakpoints -= 1;
                ()
            }).ok_or(())
        } else {
            Err(())
        }
    }

    fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS] {
        self.breakpoints
    }

    // TODO: breakpoints and watchpoints look macroable
    fn set_memory_watchpoint(&mut self, addr: Addr) -> Result<usize, ()> {
        let current_val = self.read_word(addr); // TODO: is read_word okay? What if this is a mem mapped address.
        // TODO: maybe move this into the right spot and fix the borrow checker issues.

        // Scan for the next open slot:
        for (idx, wp) in self.watchpoints.iter_mut().enumerate() {
            if let Some((a, _)) = wp {
                // For each watchpoint, check if it's already for the memory
                // location we want (note: doesn't increment the count):
                if addr == *a {
                    return Ok(idx)
                }
            } else {
                // If we reach an empty slot, use it.
                self.num_set_watchpoints += 1;
                *wp = Some((addr, current_val));
                return Ok(idx)
            }
        }

        Err(())
    }

    fn unset_memory_watchpoint(&mut self, idx: usize) -> Result<(), ()> {
        if idx < MAX_MEMORY_WATCHPOINTS {
            self.watchpoints[idx].take().map(|_| {
                // If we actually removed a watchpoint, subtract the count:
                self.num_set_watchpoints -= 1;
                ()
            }).ok_or(())
        } else {
            Err(())
        }
    }

    fn get_memory_watchpoints(&self) -> [Option<(Addr, Word)>; MAX_MEMORY_WATCHPOINTS] {
        self.watchpoints
    }

    fn run_until_event(&mut self) -> <Self as Control>::EventFuture {
        //! Note: the same batching rules that apply to the shared state apply here (see
        //! [`SharedStateState`]; basically `S` controls how this handles multiple
        //! active `run_until_event` Futures and for now `SharedStateState` is what we
        //! are using).
        let s = self.shared_state.expect("The Simulator must be provided with a shared \
            state instance if `run_until_event` is to be used.");

        self.state = State::RunningUntilEvent;

        s.add_new_future().expect("no new futures once a batch starts to resolve");

        // println!("new sim future registered!");
        EventFuture::new(s)
    }

    fn tick(&mut self) -> usize {
        // We've got a tradeoff!
        //
        // Higher values for this constant will result in better throughput while
        // lower values will improve response times.
        const STEPS_IN_A_TICK: usize = 100; // TODO: tune!

        use State::*;

        if let RunningUntilEvent = self.get_state() {
            // TODO: does this weird micro optimization help?
            if self.num_set_watchpoints == 0 && self.num_set_breakpoints == 0 {
                for _ in 0..STEPS_IN_A_TICK {
                    // this is safe since overshooting (calling step when we have NOPs) is fine
                    self.step();
                }

                return STEPS_IN_A_TICK;
            }

            for _ in 0..STEPS_IN_A_TICK {
                if let Some(e) = self.step() {
                    // If we produced some event, we're no longer `RunningUntilEvent`.
                    return STEPS_IN_A_TICK; // this is not accurate but this is allowed
                }
            }
        }

        0
    }

    fn step(&mut self) -> Option<Event> {
        use State::*;
        let current_machine_state = self.interp.step();
        let (new_state, event) = (|m: MachineState| match m {
            MachineState::Halted => {
                // If we're halted, we can't have hit a breakpoint or a watchpoint.
                (Halted, Some(Event::Halted))
            }
            MachineState::Running => {
                // Check for breakpoints:
                // (Note that if a breakpoint and a watchpoint occur at the same time,
                // the breakpoint takes precedence)
                if self.num_set_breakpoints > 0 {
                    let pc = self.get_pc();
                    if let Some(addr) = self.breakpoints.iter().filter_map(|b| *b).filter(|a| *a == pc).next() {
                        return (Paused, Some(Event::Breakpoint { addr }));
                    }
                }

                // And watchpoints:
                if self.num_set_watchpoints > 0 {
                    for i in 0..self.watchpoints.len() {
                        if let Some((addr, old_val)) = self.watchpoints[i] {
                            let data = self.read_word(addr);

                            if data != old_val {
                                self.watchpoints[i] = Some((addr, data));
                                return (Paused, Some(Event::MemoryWatch { addr, data }));
                            }
                        }
                    }

                    // if let Some((addr, data)) = self.watchpoints.iter().filter_map(|w| *w).filter(|(addr, val)| {
                    //     let current_val = self.read_word(*addr);
                    //     if current_val != *val {
                    //         *val = current_val;
                    //         true
                    //     } else { false }
                    // }).next() {
                    //     return (Paused, Some(Event::MemoryWatch { addr, data }));
                    // }
                }

                // If we didn't hit a breakpoint/watchpoint, the state doesn't change.
                // If we were running, we're still running.
                // If we were halted before, we're still halted (handled above).
                // If we were paused, we're still paused.
                (self.get_state(), None)
            }
        })(current_machine_state);

        let current_state = self.get_state();
        match (current_state, new_state, event) {
            // (RunningUntilEvent, RunningUntilEvent, Some(e)) => unreachable!(),
            // (RunningUntilEvent, RunningUntilEvent, None) => regular,
            // (RunningUntilEvent, Paused, Some(e)) => resolve,
            // (RunningUntilEvent, Paused, None) => unreachable!(), // can't stop running without an event
            // (RunningUntilEvent, Halted, Some(e)) => resolve,
            // (RunningUntilEvent, Halted, None) => unreachable!(), // can't stop running without an event

            // (Paused, RunningUntilEvent, Some(e)) => unreachable!(),    // can't start running until an event in this function
            // (Paused, RunningUntilEvent, None) => unreachable!(),       // can't start running until an event in this function
            // (Paused, Paused, Some(e)) => regular,
            // (Paused, Paused, None) => regular,
            // (Paused, Halted, Some(e)) => regular,
            // (Paused, Halted, None) => unreachable!(), // this is fine but will never happen as impl'ed above

            // (Halted, RunningUntilEvent, Some(e)) => unreachable!(),    // can't start running until an event in this function
            // (Halted, RunningUntilEvent, None) => unreachable!(),       // can't start running until an event in this function
            // (Halted, Paused, Some(e)) => unreachable!(),            // can't transition out of halted in this function
            // (Halted, Paused, None) => unreachable!(),               // can't transition out of halted in this function
            // (Halted, Halted, Some(e)) => regular,
            // (Halted, Halted, None) => unreachable!(), // this is fine but will never happen as impl'ed above

            (RunningUntilEvent, Paused, Some(e)) |
            (RunningUntilEvent, Halted, Some(e @ Event::Halted)) => {
                // println!("resolving the device future");
                self.shared_state.as_ref().expect("unreachable; must have a shared state to call a run_until_event and therefore be in `RunningUntilEvent`").resolve_all(e).unwrap();
                self.state = new_state;
                Some(e)
            },

            (RunningUntilEvent, RunningUntilEvent, e @ None) |
            (Paused, Paused, e @ Some(_))                    |
            (Paused, Paused, e @ None)                       |
            (Paused, Halted, e @ Some(Event::Halted))        |
            (Halted, Halted, e @ Some(Event::Halted)) => {
                self.state = new_state;
                e
            }

            (RunningUntilEvent, Halted, Some(_)) |
            (Paused, Halted, Some(_))            |
            (Halted, Halted, Some(_)) => unreachable!("Transitions to the `Halted` state must only produce halted events."),

            (RunningUntilEvent, RunningUntilEvent, Some(_)) => unreachable!("Can't yield an event and not finish a `RunningUntilEvent`."),

            (RunningUntilEvent, Paused, None) |
            (RunningUntilEvent, Halted, None) => unreachable!("Can't finish a `RunningUntilEvent` without an event."),

            (Paused, RunningUntilEvent, Some(_)) |
            (Paused, RunningUntilEvent, None)    |
            (Halted, RunningUntilEvent, Some(_)) |
            (Halted, RunningUntilEvent, None) => unreachable!("Can't start a 'run until event' in this function."),

            (Halted, Paused, Some(_)) |
            (Halted, Paused, None) => unreachable!("Can't get out of the `Halted` state in this function."),

            (Paused, Halted, None) |
            (Halted, Halted, None) => unreachable!("Always produce an event when the next state is `Halted`."),
        }
    }

    fn pause(&mut self) {
        use State::*;

        match self.get_state() {
            Halted | Paused => {}, // Nothing changes!
            RunningUntilEvent => {
                self.shared_state.as_ref().expect("unreachable; must have a shared state to call a run_until_event and therefore be in `RunningUntilEvent`").resolve_all(Event::Interrupted);
                self.state = Paused;
            }
        }
    }

    fn get_state(&self) -> State {
        self.state
    }

    fn reset(&mut self) {
        self.interp.halt();

        // Resolve all futures! Doesn't cause problems if reset is called
        // multiple times.
        let _ = self.step();

        InstructionInterpreter::reset(&mut self.interp);
        self.state = State::Paused;

        // For now, we won't force all futures to have resolved on a reset.
        // We're still calling reset here (currently a no-op) because eventually
        // this should advance the batch counter (though that may happen
        // on set_event, rendering this function entirely unnecessary).
        if let Some(s) = self.shared_state {
            s.reset()
        }
    }

    fn get_error(&self) -> Option<Error> {
        unimplemented!()
    }

    fn get_gpio_states(&self) -> GpioPinArr<GpioState> {
        Gpio::get_states(self.interp.get_peripherals())
    }

    fn get_gpio_readings(&self) -> GpioPinArr<Result<bool, GpioReadError>> {
        Gpio::read_all(self.interp.get_peripherals())
    }

    fn get_adc_states(&self) -> AdcPinArr<AdcState> {
        Adc::get_states(self.interp.get_peripherals())
    }

    fn get_adc_readings(&self) -> AdcPinArr<Result<u8, AdcReadError>> {
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

    fn get_info(&self) -> DeviceInfo {
        DeviceInfo::new(
            self.interp.get_program_metadata(),
            Default::default(), // TODO: when we add other capabilities
            I::type_id(),
            self.id(),
            Default::default(), // no proxies (yet)
        )
    }

    fn set_program_metadata(&mut self, metadata: ProgramMetadata) {
        self.interp.set_program_metadata(metadata)
    }

    fn id(&self) -> Identifier {
        I::ID
    }
}

// #[derive(Debug)]
// pub struct SimFuture<'S, S: EventFutureSharedStatePorcelain>(&'s S);

// impl<'s> Future for SimFuture<'s> {
//     type Output = (Event, State);

//     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {

//     }
// }
