//! Device side for the [`Control`](super::Control) RPC set up.
//!
//! TODO!

// TODO: auto gen (proc macro, probably) the crimes below from the `Control`
// trait.

use super::{Encode, Decode, Transport};
use super::{Control, RequestMessage, ResponseMessage};
use super::encoding::Transparent;

use core::marker::PhantomData;
use core::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
use core::future::Future;
use core::pin::Pin;
use core::fmt::Debug;

// Check for messages and execute them on something that implements the [`Control`]
// interface.
//
// Receives Requests and sends Responses.
//
// As mentioned elsewhere, there's a level of indirection of
// `RequestMessage`/`ResponseMessage` and the types used here so that users can
// experiment with their own messages. This may, however, be moot (TODO). See
// the docs in `controller.rs` for more info.
#[derive(Debug, Default)]
pub struct Device<
    T,
    C,
    Req = RequestMessage,
    Resp = ResponseMessage,
    ReqDec = Transparent<RequestMessage>,
    RespEnc = Transparent<ResponseMessage>,
>
where
    Req: Debug,
    Resp: Debug,
    Req: Into<RequestMessage>,
    ResponseMessage: Into<Resp>,
    ReqDec: Decode<Req>,
    RespEnc: Encode<Resp>,
    T: Transport<<RespEnc as Encode<Resp>>::Encoded, <ReqDec as Decode<Req>>::Encoded>,
    C: Control,
    // <C as Control>::EventFuture: Unpin,
{
    _encoding: PhantomData<(Req, Resp, ReqDec, RespEnc)>,
    _control_impl: PhantomData<C>,
    pub transport: T,
    // pending_event_future: Option<Pin<C::EventFuture>>,
    pending_event_future: Option<C::EventFuture>,
}

// TODO: make a builder!

impl<Req, Resp, D, E, T, C> Device<T, C, Req, Resp, D, E>
where
    Req: Debug,
    Resp: Debug,
    Req: Into<RequestMessage>,
    ResponseMessage: Into<Resp>,
    D: Decode<Req>,
    E: Encode<Resp>,
    T: Transport<<E as Encode<Resp>>::Encoded, <D as Decode<Req>>::Encoded>,
    C: Control,
    // <C as Control>::EventFuture: Unpin,
{
    // When const functions can be in blanket impls, this can be made `const`.
    //
    // Note: we take `decode` and `encode` as parameters here even though the
    // actual value is never used so that users don't have to resort to using
    // the turbofish syntax to specify what they want the encoding layer to be.
    pub /*const*/ fn new(_enc: E, _dec: D, transport: T) -> Self {
        Self {
            // encoding,
            _encoding: PhantomData,
            _control_impl: PhantomData,
            transport,
            pending_event_future: None,
        }
    }
}

// This is meant to be a barebones blocking executor.
// It has been updated to more or less mirror the 'executor' in
// [`genawaiter`](https://docs.rs/crate/genawaiter/0.2.2/source/src/waker.rs);
// while this doesn't guarantee correctness, it does make me feel a little
// better.
//
// Another option would have been to use (or steal) `block_on` from the
// `futures` crate. We may switch to doing this in the future but currently we
// don't because:
//   - really, we want to _poll_ the future in the step function and not block
//     on it
//   - because the future that the simulator (i.e. the Control implementor)
//     that's given to a Device instance is ostensibly an EventFuture, it's
//     not going to need to do any real I/O; we can absolutely get away with
//     a fake executor that doesn't have a reactor and never actually does
//     any scheduling
//
// This is fine for now, but is definitely not ideal. If, in the future, we
// write simulators that do real async I/O and produce real Futures, we may need
// to use an executor from `tokio` (a cursory glance through the executors in
// `futures` seems to suggest they don't do anything special to accommodate I/O
// but I'm probably wrong). IIUC, this is mostly a performance thing; without an
// actual scheduler we'll just be blindly calling poll far more often than we
// actually should.
//
// I'm okay with this for now, since:
//   - it's not super apparent to me what an actual async API for the simulator
//     (i.e. an async version of Control) would look like
//   - we can't do async in Traits yet (_properly_) anyways
//
// (Glancing through `async-std` seems to confirm that the executor, the reactor
// and the futures are somewhat decoupled: the executor bumbles along trying to
// finish tasks and very politely asking futures to let it know when tasks that
// are blocked will become unblocked (i.e. the Waker API); futures are to call
// out to the actual hardware to do work and also to _arrange_ for the executor
// to be notified when they can make further process; the reactor is the thing
// that arranges for whatever is underneath us to alert the executor when a
// certain task can make progress. This suggests that individual futures can be
// somewhat tied to reactor implementations. For example, `async-std` uses `mio`
// and would not function correctly if the futures its functions produce were
// executed without a Mio based reactor being present; instead the futures would
// report that they were `NotReady`, would try to register themselves with their
// Mio based reactor (passing along their context or just their Waker) and would
// then be queued by their executor, unaware that no arrangement had actually
// been made to inform the executor once they could be awoken. All this is to
// say that futures and their reactor definitely are coupled and it seems like
// an executor must either start a reactor thread (that futures then somehow
// communicate with??) or there must be some way to guarantee that a globally
// accessible reactor instance will be available and running. Again, I've only
// glanced through async-std but it seems to opt for something like the latter;
// Reactor instances (per thing, I think -- like net has it's own reactor) are
// global and -- cleverly -- only started once the global variable the instance
// lives in is accessed. This lets the executor be largely unaware of the
// reactor, I think. Still not totally clear on how the futures interact with
// Mio (the reactor) but it has to do with `Evented` and the types Mio exposes.
// I have basically not looked at `tokio` at all but I think it does something
// fancier with layers (the executor, some timers, and the reactor are layers,
// in that order, I think)).
//
// [Reactor]: {Registers events w/kernel}  <-----
//    (proxies) | (wake these)                   \
//              | ( futures  )                   |
//              |               ----> {Register Wake} -> [Sleep]
//              v              /    ^----------------|
// future -> [Executor] -> Task-----> [Running] -----|
//                             \    ^-v--------------|
//                              ------> [Finished]
//
//
// Sidenote: the desire for these pieces to be decoupled is why the Waker API
// is as "build it yourself" as it is iiuc (i.e. the raw vtable); they couldn't
// do a trait because then everything would have to be generic over the Waker;
// they couldn't do a trait object because then object safety rears its head
// (associated types? i think) so: all type safety was sacrificed. iirc one of
// withoutboats' async blog posts talks about it.
//
// Anyways. Now that we've got something of an understanding, we can talk about
// our fairly simple use case.
//
// For clarity, here's our whole picture (same as in the [module docs](super)):
//
// ```
//   /----------------------------------------------------------------------\
//  |                    [Controller Side: i.e. Laptop]                      |
//  |                                                                        |
//  |  /----------------------\                     %%% < %%%                |
//  | | [Controller]: `Control`|               %%% [Main Loop] %%%           |
//  | | tick:                  |                                             |
//  | |  - resolves futures    |           %%%  /---------------\  %%%       |
//  | |    issued by           |               |  [Client Logic] |           |
//  | |    `run_until_event`   |<---\     %%%  |                 |   %%%     |
//  | | rest:                  |    |     vvv  | Uses the device |   ^^^     |
//  | |  - proxied; send req.  |    |     %%%  | via the Control |   %%%     |
//  | |    and block on resp.  |    |          | interface.      |           |
//  |  \--|----------------^--/     |     %%%  |  /---^          |  %%%      |
//  |     |                |        |           \-|-------------/            |
//  | |---v----|     |-----|---|    |        %%%  v              %%%         |
//  | |Enc: Req|     |Dec: Resp|    \----------->[Control::tick]             |
//  | |-|------|     |-------^-|                    %%% > %%%                |
//   \--|--------------------|----------------------------------------------/
//      |<Con Send  Con Recv>|
//      |  [Transport Layer] |
//      |<Dev Recv  Dev Send>|
//   /--v--------------------|----------------------------------------------\
//  | |--------|     |-------|-|            %%% < %%%            /--------\  |
//  | |Dec: Req|     |Enc: Resp|       %%% [Dev. Loop] %%%      |  [Sim.]  | |
//  | |---|----|     |-----^---|                       /--------| ╭──────╮ | |
//  |     |                |       %%%                 |   %%%  | │Interp│ | |
//  |  /--v----------------|--\                        |        | ╰──────╯ | |
//  | |        [Device]        |  %%%                  v     %%% \--------/  |
//  | | tick:                  |  vvv [Device::tick(......)] ^^^             |
//  | |  - makes progress on   |  %%%     |                  %%%             |
//  | |    any futures that    |<---------/                                  |
//  | |    were issued         |  %%%                       %%%              |
//  | |  - processes new reqs  |                                             |
//  | |    (blocks if not a    |     %%%  v              %%%                 |
//  | |    `run_until_event`)  |                                             |
//  |  \----------------------/             %%% > %%%                        |
//  |                                                                        |
//  |                         [Device Side: i.e. TM4C]                       |
//   \----------------------------------------------------------------------/
// ```

// ╭──────╮
// │Interp│
// ╰──────╯
// ╔──────╗
// │Interp│
// ╚──────╝
// ╔══════╗
// ║Interp║
// ╚══════╝

static NO_OP_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    RW_CLONE,
    RW_WAKE,
    RW_WAKE_BY_REF,
    RW_DROP
);

#[doc(hidden)]
pub static RW_CLONE: fn(*const ()) -> RawWaker = |_| RawWaker::new(
    core::ptr::null(),
    &NO_OP_RAW_WAKER_VTABLE,
);
static RW_WAKE: fn(*const ()) = |_| { };
static RW_WAKE_BY_REF: fn(*const ()) = |_| { };
static RW_DROP: fn(*const ()) = |_| { };

impl<Req, Resp, D, E, T, C> Device<T, C, Req, Resp, D, E>
where
    Req: Debug,
    Resp: Debug,
    Req: Into<RequestMessage>,
    ResponseMessage: Into<Resp>,
    D: Decode<Req>,
    E: Encode<Resp>,
    T: Transport<<E as Encode<Resp>>::Encoded, <D as Decode<Req>>::Encoded>,
    C: Control,
    <C as Control>::EventFuture: Unpin, // TODO: use `pin_utils::pin_mut!` and relax this requirement.
    // <C as Control>::EventFuture: Deref<Target = <C as Control>::EventFuture>,
    // <C as Control>::EventFuture: Deref,
    // <C as Control>::EventFuture: DerefMut,
    // <<C as Control>::EventFuture as Deref>::Target: Future<Output = (Event, State)>,
    // <<C as Control>::EventFuture as Deref>::Target: Unpin,
{
    #[allow(unsafe_code)]
    pub fn step(&mut self, c: &mut C) -> usize {
        use RequestMessage::*;
        use ResponseMessage as R;
        let mut num_processed_messages = 0;

        // Make some progress:
        c.tick();

        if let Some(ref mut f) = self.pending_event_future {
            // println!("polling the device future");

            // TODO: we opt to poll here because we assume that the underlying future is
            // rubbish so our waker (if we were to register a real one) would never be called.
            //
            // However, the simulator's future (just as the one the controller exposes) does
            // treat the waker correctly. Additionally if someone were to write a truly
            // async simulator, this would also be a real future that respects the waker.
            //
            // So, it may be worth looking into using a real waker that notifies us that
            // something has happened. Or better yet, maybe writing an async Device rpc
            // thing that just chains our future onto the real one.
            //
            // On the other hand, this is at odds with no_std support and it's unlikely
            // to net material performance wins so, maybe eventually.
            if let Poll::Ready(event) = Pin::new(f).poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RW_CLONE(&())) } )) {
                // println!("device future is done!");
                self.pending_event_future = None;

                let enc = E::encode(R::RunUntilEvent(event).into());
                self.transport.send(enc).unwrap(); // TODO: don't panic?
            }
        }

        while let Ok(m) = self.transport.get().map(|enc| D::decode(&enc).unwrap().into()) {
            num_processed_messages += 1;

            macro_rules! dev {
                ($(($req:pat => $($resp:tt)+) with $r:tt = $resp_expr:expr;)*) => {
                    #[forbid(unreachable_patterns)]
                    match m {
                        RunUntilEvent => {
                            if self.pending_event_future.is_some() {
                                panic!() // TODO: write a message // already have a run until event pending!
                            } else {
                                // self.pending_event_future = Some(Pin::new(c.run_until_event()));
                                self.pending_event_future = Some(c.run_until_event());
                                self.transport.send(E::encode(R::RunUntilEventAck.into())).unwrap()
                            }
                        },
                        $(
                            $req => self.transport.send(E::encode({
                                let $r = $resp_expr;
                                $($resp)+
                            }.into())).unwrap(),
                        )*
                    }

                };
            }

            dev! {
                (GetPc => R::GetPc(r)) with r = c.get_pc();
                (SetPc { addr } => R::SetPc) with _ = c.set_pc(addr);

                (GetRegister { reg } => R::GetRegister(r)) with r = c.get_register(reg);
                (SetRegister { reg, data } => R::SetRegister) with _ = c.set_register(reg, data);

                (GetRegistersPsrAndPc => R::GetRegistersPsrAndPc(r)) with r = c.get_registers_psr_and_pc();

                (ReadWord { addr } => R::ReadWord(r)) with r = c.read_word(addr);
                (WriteWord { addr, word } => R::WriteWord) with _ = c.write_word(addr, word);

                (StartPageWrite { page, checksum } => R::StartPageWrite(r)) with r = c.start_page_write(page, checksum);
                (SendPageChunk { offset, chunk } => R::SendPageChunk(r)) with r = c.send_page_chunk(offset, chunk);
                (FinishPageWrite { page } => R::FinishPageWrite(r)) with r = c.finish_page_write(page);

                (SetBreakpoint { addr } => R::SetBreakpoint(r)) with r= c.set_breakpoint(addr);
                (UnsetBreakpoint { idx } => R::UnsetBreakpoint(r)) with r = c.unset_breakpoint(idx);
                (GetBreakpoints => R::GetBreakpoints(r)) with r = c.get_breakpoints();
                (GetMaxBreakpoints => R::GetMaxBreakpoints(r)) with r = c.get_max_breakpoints();

                (SetMemoryWatchpoint { addr } => R::SetMemoryWatchpoint(r)) with r = c.set_memory_watchpoint(addr);
                (UnsetMemoryWatchpoint { idx } => R::UnsetMemoryWatchpoint(r)) with r = c.unset_memory_watchpoint(idx);
                (GetMemoryWatchpoints => R::GetMemoryWatchpoints(r)) with r = c.get_memory_watchpoints();
                (GetMaxMemoryWatchpoints => R::GetMaxMemoryWatchpoints(r)) with r = c.get_max_memory_watchpoints();

                (Step => R::Step(r)) with r = c.step();
                (Pause => R::Pause) with _ = c.pause();
                (GetState => R::GetState(r)) with r = c.get_state();
                (Reset => R::Reset) with _ = c.reset();

                (GetError => R::GetError(r)) with r = c.get_error();

                (GetGpioStates => R::GetGpioStates(r)) with r = c.get_gpio_states();
                (GetGpioReadings => R::GetGpioReadings(r)) with r = c.get_gpio_readings();

                (GetAdcStates => R::GetAdcStates(r)) with r = c.get_adc_states();
                (GetAdcReadings => R::GetAdcReadings(r)) with r = c.get_adc_readings();

                (GetTimerStates => R::GetTimerStates(r)) with r = c.get_timer_states();
                (GetTimerConfig => R::GetTimerConfig(r)) with r = c.get_timer_config();

                (GetPwmStates => R::GetPwmStates(r)) with r = c.get_pwm_states();
                (GetPwmConfig => R::GetPwmConfig(r)) with r = c.get_pwm_config();

                (GetClock => R::GetClock(r)) with r = c.get_clock();

                (GetDeviceInfo => R::GetDeviceInfo(r)) with r = c.get_device_info().add_proxy(T::ID).expect("too many proxies");

                (GetProgramMetadata => R::GetProgramMetadata(r)) with r = c.get_program_metadata();
                (SetProgramMetadata { metadata } => R::SetProgramMetadata) with _ = c.set_program_metadata(metadata);
            };
        }

        num_processed_messages
    }
}
