//! TODO!

use super::BlackBox;
use crate::{
    init::Init,
    shim_support::{new_shim_peripherals_set, ShimPeripheralSet, Shims},
};

use lc3_baseline_sim::{
    interp::{
        InstructionInterpreter, Interpreter, InterpreterBuilder,
        PeripheralInterruptFlags,
    },
    sim::Simulator,
};
use lc3_shims::{memory::MemoryShim, peripherals::SourceShim};
use lc3_traits::control::{rpc::futures::SyncEventFutureSharedState, Control};

use std::sync::Mutex;

// Static data that we need:
lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE: SyncEventFutureSharedState =
        SyncEventFutureSharedState::new();
}

static FLAGS: PeripheralInterruptFlags = PeripheralInterruptFlags::new();

type Interp<'io> =
    Interpreter<'static, MemoryShim, ShimPeripheralSet<'static, 'io>>;
pub(crate) type Sim<'io> =
    Simulator<'static, 'static, Interp<'io>, SyncEventFutureSharedState>;

pub(crate) fn new_sim<'io>(shims: ShimPeripheralSet<'static, 'io>) -> Sim<'io> {
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

    let mut sim: Sim<'io> =
        Simulator::new_with_state(interp, &*EVENT_FUTURE_SHARED_STATE);
    sim.reset();

    sim
}

pub struct SimDevice<'io> {
    sim: Option<Sim<'io>>,
    input: Option<SourceShim>,
    output: Option<Mutex<Vec<u8>>>,
}

// Ultimately, because we can't do self referential structs in safe Rust, we
// have to resort to making the input/output sink/source 'static. :-/
//
// This basically means leaking memory which actually isn't too terrible in this
// case.
impl<'s> Init<'s> for SimDevice<'static> {
    type Config = ();

    type ControlImpl = Sim<'static>;
    type Input = SourceShim;
    type Output = Mutex<Vec<u8>>;

    fn init(
        b: &'s mut BlackBox,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    ) {
        // This is extremely unsafe and almost certainly UB.
        //
        // Even though input and output do live as long as the simulator (since
        // we never drop the input and output without also dropping the
        // simulator and since we don't give out mutable references to input
        // and output) it seems very hard to _prove_ this is true.
        //
        // Even if it is, faking a 'static reference here might be UB for other
        // reasons.
        /*
        let input = Some(SourceShim::new());
        let output = Some(Mutex::new(Vec::new()));
        let storage: &'s mut _ = b.put::<_>(SimStorage { sim: None, input, output });

        let inp = storage.input.as_ref().unwrap();
        let out = storage.output.as_ref().unwrap();

        let input = unsafe { core::mem::transmute::<&'s Self::Input, &'static Self::Input>(inp) };
        let output = unsafe { core::mem::transmute::<&'s Self::Output, &'static Self::Output>(out) };
        */

        // Meanwhile, this is safe but leaks memory 🙁.
        /*  */
        let storage: &'s mut _ = b.put(SimDevice {
            sim: None,
            input: None,
            output: None,
        });

        let inp: &'static SourceShim = Box::leak(Box::new(SourceShim::new()));
        let out: &'static Mutex<Vec<u8>> =
            Box::leak(Box::new(Mutex::new(Vec::new())));

        let input = inp;
        let output = out;
        /*  */

        let (shims, _, _) =
            new_shim_peripherals_set::<'static, 'static, _, _>(input, output);
        let shim_copy = Shims::from_peripheral_set(&shims);

        storage.sim = Some(new_sim(shims));

        (
            storage.sim.as_mut().unwrap(),
            Some(shim_copy),
            Some(inp),
            Some(out),
        )
    }
}
