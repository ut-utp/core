//! Common things between the benchmarks.

pub extern crate lc3_baseline_sim;
pub extern crate lc3_isa;
pub extern crate lc3_shims;
pub extern crate lc3_traits;
pub extern crate lc3_os;

extern crate lazy_static;

use lc3_isa::{program, util::AssembledProgram};
pub use lc3_isa::Word;

use lc3_os::OS_IMAGE;

pub const fn fib_program_executed_insn_count(num_iters: Word) -> u64 {
    (159 * (num_iters as u64) + 347)
}

// TODO: new macro that basically does the below + sets the orig hook
// TODO: have obj conv set the orig hook

pub fn build_fib_memory_image(num_iters: Word) -> MemoryDump {
    const F: Word = 24;

    let prog: AssembledProgram = program! {
        .ORIG #0x3000;
        BRnzp @START;

        @NUM_ITERS .FILL #num_iters;
        @FIB_NUM .FILL #F;

        @START
        LD R1, @NUM_ITERS;

        @LOOP
            BRz @END;

            @FIB_START
                AND R3, R3, #0; // R3 = 0
                ADD R4, R3, #1; // R4 = 1

                LD R2, @FIB_NUM;

            @FIB
                // ADD R2, R2, #0;
                BRz @END_FIB;

                ADD R5, R3, #0;
                ADD R3, R4, #0;
                ADD R4, R4, R5;

                ADD R2, R2, #-1;
                BRnzp @FIB;

            @END_FIB
                ADD R0, R3, #0;
                OUT;

            ADD R1, R1, #-1;
            BRnzp @LOOP;

        @END
            HALT;
    }
    .into();

    let mut image = OS_IMAGE.clone();
    image.layer_loadable(&prog);

    image
}

pub fn fib_closed_form(n: Word) -> u64 {
    let g: f64 = (1. + 5f64.sqrt()) / 2.0;
    let r: f64 = (g.powi(n as i32) - (-g).powi(-(n as i32))) / 5f64.sqrt();

    r as u64
}

use self::lazy_static::lazy_static;

use lc3_baseline_sim::interp::{Interpreter, InterpreterBuilder};
pub use lc3_baseline_sim::interp::{InstructionInterpreter, PeripheralInterruptFlags};
use lc3_isa::util::MemoryDump;
use lc3_shims::{memory::MemoryShim};
use lc3_traits::control::{
    rpc::{Controller, Device, Transport, Decode, Encode},
};
pub use lc3_traits::control::Control;

use lc3_traits::peripherals::stubs::PeripheralsStub;

pub fn bare_interpreter<'a, 'b>(
    program: MemoryDump,
    flags: &'b PeripheralInterruptFlags,
) -> Interpreter<'b, MemoryShim, PeripheralsStub<'b>> {
    let memory = MemoryShim::new(*program);

    let mut interp: Interpreter<'b, MemoryShim, PeripheralsStub<'b>> = InterpreterBuilder::new()
        .with_defaults()
        .with_memory(memory)
        .build();

    interp.reset();
    interp.init(flags);

    interp
}

use lc3_baseline_sim::sim::Simulator;
use lc3_traits::control::rpc::SyncEventFutureSharedState;

lazy_static! {
    static ref SIM_STATE: SyncEventFutureSharedState = SyncEventFutureSharedState::new();
}

type Sim<'a> = Simulator<'a, 'static, Interpreter<'a, MemoryShim, PeripheralsStub<'a>>, SyncEventFutureSharedState>;

pub fn simulator<'a>(program: MemoryDump, flags: &'a PeripheralInterruptFlags) -> Sim<'a> {
    let mut sim = Simulator::new_with_state(bare_interpreter(program, flags), &*SIM_STATE);
    sim.reset();

    sim
}

use std::thread::Builder as ThreadBuilder;

pub static FLAGS: PeripheralInterruptFlags = PeripheralInterruptFlags::new();
fn device_thread<ReqDec: 'static, RespEnc: 'static, Transp: 'static>(
    rx: Receiver<()>,
    mut device: Device<Transp, Sim<'static>, RequestMessage, ResponseMessage, ReqDec, RespEnc>,
    program: MemoryDump,
) where
    ReqDec: Decode<RequestMessage> + Send,
    RespEnc: Encode<ResponseMessage> + Send,
    Transp: Transport<RespEnc::Encoded, ReqDec::Encoded> + Send,
{
    ThreadBuilder::new()
        .name("Device Thread".to_string())
        .stack_size(1024 * 1024 * 4)
        .spawn(move || {
            let mut sim = simulator(program, &FLAGS);

            loop {
                device.step(&mut sim);
                sim.tick();
                if let State::Halted = sim.get_state() {
                    if let Ok(()) = rx.try_recv() {
                        break;
                    }
                }
            }
        });
}

lazy_static! {
    static ref RPC_STATE: SyncEventFutureSharedState = SyncEventFutureSharedState::new();
}

use lc3_traits::control::rpc::{mpsc_sync_pair, MpscTransport, ResponseMessage, RequestMessage};
use lc3_traits::control::rpc::encoding::Transparent;
use std::sync::mpsc::{channel, Receiver, Sender};

// TODO: test spin vs. sleep
pub fn remote_simulator/*<C: Control>*/(program: MemoryDump) -> (Sender<()>, Controller<'static, MpscTransport<RequestMessage, ResponseMessage>, SyncEventFutureSharedState>)
// where
//     <C as Control>::EventFuture: Sync + Send,
{
    let (controller, device) = mpsc_sync_pair::<_, _, _, _, Transparent<_>, Transparent<_>, _>(&RPC_STATE);
    let (tx, rx) = channel();

    device_thread(rx, device, program);

    (tx, controller)
}


use lc3_traits::control::State;

pub fn executor_thread<C: Control>(mut dev: C) -> (Sender<Option<()>>, impl Fn(&Sender<Option<()>>), impl Fn(&Sender<Option<()>>) -> C::EventFuture)
where
    C: Send + 'static,
    <C as Control>::EventFuture: Send,
{
    let (halt_or_fut, rx_halt_or_fut) = channel();
    let (tx_fut, rx_fut) = channel();
    std::thread::spawn(move || {
        loop {
            match rx_halt_or_fut.try_recv() {
                Err(_) => { dev.tick(); },
                Ok(None) => break,
                Ok(Some(())) => {
                    dev.reset();
                    tx_fut.send(dev.run_until_event()).unwrap();
                }
            }
        }
    });

    let next = move |c: &Sender<Option<()>>| { c.send(Some(())).unwrap(); rx_fut.recv().unwrap() };
    let halt = |c: &Sender<Option<()>>| c.send(None).unwrap();

    (halt_or_fut, halt, next)
}
