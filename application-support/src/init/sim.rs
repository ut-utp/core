//! TODO!

use crate::init::Init;
use crate::shim_support::{Shims, ShimPeripheralSet, new_shim_peripherals_set};
use super::BlackBox;

use lc3_shims::memory::MemoryShim;
use lc3_shims::peripherals::SourceShim;
use lc3_baseline_sim::interp::{InstructionInterpreter, Interpreter, InterpreterBuilder, PeripheralInterruptFlags};
use lc3_baseline_sim::sim::Simulator;
use lc3_traits::control::rpc::futures::SyncEventFutureSharedState;
use lc3_traits::control::Control;

use std::sync::Mutex;

// Static data that we need:
lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE: SyncEventFutureSharedState = SyncEventFutureSharedState::new();
}

static FLAGS: PeripheralInterruptFlags = PeripheralInterruptFlags::new();

type Interp<'io> = Interpreter<'static, MemoryShim, ShimPeripheralSet<'static, 'io>>;
type Sim<'io> = Simulator<'static, 'static, Interp<'io>, SyncEventFutureSharedState>;

pub struct SimStorage<'io> {
    sim: Option<Sim<'io>>,
    input: SourceShim,
    output: Mutex<Vec<u8>>,
}

// Ultimately, because we can't do self referential structs in safe Rust, we have to
// resort to making the input/output sink/source 'static. :-/
//
// This basically means leaking memory which actually isn't too terrible in this case.
impl<'s> Init<'s> for SimStorage<'static> {
    type Config = ();

    type ControlImpl = Sim<'static>;
    type Input = SourceShim;
    type Output = Mutex<Vec<u8>>;

    // fn init<'a/*, 'b*/>(b: &'a mut BlackBox)
    fn init(b: &'s mut BlackBox)
            -> (&'s mut Self::ControlImpl, Option<Shims</*'b*/'static>>, Option<&'s Self::Input>, Option<&'s Self::Output>) {
        let input = SourceShim::new();
        let output = Mutex::new(Vec::new());
        let storage: &'s mut _ = b.put::<_>(SimStorage { sim: None, input, output });

        // This is extremely unsafe and almost certainly UB.
        //
        // Even though input and output do live as long as the simulator (since
        // we never drop the input and output without also dropping the
        // simulator and since we don't give out mutable references to input
        // and output) it seems very hard to _prove_ this is true.
        //
        // Even if it is, faking a 'static reference here might be UB for other
        // reasons.
        let input = unsafe { core::mem::transmute::<&'s _, &'static Self::Input>(&storage.input) };
        let output = unsafe { core::mem::transmute::<&'s _, &'static Self::Output>(&storage.output) };

        let (shims, _, _) = new_shim_peripherals_set::<'static, 'static, _, _>(input, output);
        // let (shims, _, _) = new_shim_peripherals_set::<'static, 's, _, _>(&storage.input, &storage.output);
        let shim_copy = Shims::from_peripheral_set(&shims);

        let mut interp: Interpreter<'_, _, _> = InterpreterBuilder::new()
            .with_interrupt_flags_by_ref(&FLAGS)
            .with_peripherals(shims)
            .with_default_memory()
            .with_default_regs()
            .with_default_pc()
            .with_default_state()
            .build();

        interp.reset();
        interp.init(&FLAGS);

        let mut sim: Sim<'static> = Simulator::new_with_state(interp, &*EVENT_FUTURE_SHARED_STATE);
        sim.reset();

        storage.sim = Some(sim);

        (storage.sim.as_mut().unwrap(), Some(shim_copy), Some(&storage.input), Some(&storage.output))
        // (storage.sim.as_mut().unwrap(), None, Some(&storage.input), Some(&storage.output))
    }
}
