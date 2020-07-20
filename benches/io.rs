
extern crate lc3_baseline_sim;
extern crate lc3_isa;
extern crate lc3_shims;
extern crate lc3_traits;
extern crate lc3_os;

extern crate criterion;
extern crate async_std;
extern crate lazy_static;

#[path = "common.rs"]
mod common;
use common::*;

use lc3_traits::peripherals::input::{Input, InputError};
use lc3_shims::peripherals::input::{InputShim, Source};

use std::sync::{Arc, Mutex};

pub struct BufferedInput<Iter: Iterator<Item = u8>> {
    buffer: Mutex<Iter>,
}

impl<I: Iterator<Item = u8>> BufferedInput<I> {
    pub fn new(iter: I) -> Self {
        Self {
            buffer: Mutex::new(iter),
        }
    }
}

impl<I: Iterator<Item = u8>> Source for BufferedInput<I> {
    fn get_char(&self) -> Option<u8> {
        self.buffer.lock().unwrap().next()
    }
}

use lc3_baseline_sim::interp::{Interpreter, InterpreterBuilder};
use lc3_traits::peripherals::{stubs::{GpioStub, AdcStub, PwmStub, TimersStub, ClockStub}, PeripheralSet};
use lc3_shims::peripherals::output::{OutputShim, Sink};
use lc3_shims::memory::MemoryShim;
use lc3_isa::util::MemoryDump;

pub fn interpreter<'b, 's>(
    program: &MemoryDump,
    flags: &'b PeripheralInterruptFlags,
    inp: impl Iterator<Item = u8> + Send + 's,
    out: impl Sink + Send + Sync + 's,
) -> Interpreter<
    'b,
    MemoryShim,
    PeripheralSet<
        'b,
        GpioStub,
        AdcStub,
        PwmStub,
        TimersStub,
        ClockStub,
        InputShim<'s, 'b>,
        OutputShim<'s, 'b>,
    >
> {
    let memory = MemoryShim::new(**program);

    let peripherals = PeripheralSet::new(
        GpioStub,
        AdcStub,
        PwmStub,
        TimersStub,
        ClockStub,
        InputShim::using(Box::new(BufferedInput::new(inp))),
        OutputShim::using(Box::new(out)),
    );

    let mut interp: Interpreter::<'b, MemoryShim, _> = InterpreterBuilder::new()
        .with_defaults()
        .with_peripherals(peripherals)
        .with_memory(memory)
        .build();

    interp.reset();
    interp.init(flags);

    interp
}

fn byte_stream(elements: usize) -> impl Clone + Iterator<Item = u8> {
    (0..elements)
        .map(|i| (((i % 256) + (i * i * i) - (i + 12)) % 256) as u8)
        // lc3tools remaps 13s to 10s (i.e. '\r' → '\n') so we have to do this
        .map(|i| match i { 13 => 10, i => i })
}

fn checksum(iter: impl Iterator<Item = u8>) -> u128 {
    iter.fold(1u128, |acc, b| acc * ((b + 1) as u128))
}

pub fn program(num_elements: u64) -> AssembledProgram {
    let prog = program! {
        // Disable PUTS to suppress the HALT message.
        .ORIG #(lc3_os::traps::builtin::PUTS as Word);
        .FILL @NEW_PUTS;

        .ORIG #0x4000;
        @NEW_PUTS RTI;


        .ORIG #0x3000;
        BRnzp @START;

        // This is kinda cryptic but what's going on here is that we branch to
        // the innermost loop, skipping the zero check for the 3 outer loops
        // (we do this because if we didn't we'd exit prematurely when outer
        // loop counters are zero even when inner loop counters are still
        // non-zero).
        //
        // Because of this, we end up doing a subtract on the outer loop
        // counters immediately once we get out of the innermost loop, which
        // actually throws the count off since those loops haven't actually
        // run yet. To compensate for this we add 1 to their counts. Note that
        // is valid for Word::MAX_VALUE too (the value will start as 0 which
        // wraps to MAX_VALUE when 1 is subtracted).
        //
        // We don't do this addition to the innermost loop because it's zero
        // check actually does get run (that's our starting point).
        //
        // This is all pretty precarious but it lets us avoid lots of special
        // cases and manual checks!
        @C_A .FILL #(((num_elements >> 48) as Word).wrapping_add(1));
        @C_B .FILL #(((num_elements >> 32) as Word).wrapping_add(1));
        @C_C .FILL #(((num_elements >> 16) as Word).wrapping_add(1));
        @C_D .FILL #(((num_elements >> 00) as Word).wrapping_add(0));

        @START
        LD R1, @C_A;
        LD R2, @C_B;
        LD R3, @C_C;
        LD R4, @C_D;

        BRnzp @D_LOOP;

        @A_LOOP
            BRz @A_END;

            @B_LOOP
                BRz @B_END;

                @C_LOOP
                    BRz @C_END;

                    @D_LOOP
                        BRz @D_END;

                        GETC;
                        OUT;

                        ADD R4, R4, #-1;
                        BRnzp @D_LOOP;

                    @D_END
                        ADD R4, R4, #-1;
                        ADD R3, R3, #-1;
                        BRnzp @C_LOOP;

                @C_END
                    ADD R3, R3, #-1;
                    ADD R2, R2, #-1;
                    BRnzp @B_LOOP;

            @B_END
                ADD R2, R2, #-1;
                ADD R1, R1, #-1;
                BRnzp @A_LOOP;

        @A_END
            HALT;
    }.into();

    prog
}

pub fn raw_io_program(num_elements: u64) -> AssembledProgram {
    let prog = program! {
        // Disable PUTS to suppress the HALT message.
        .ORIG #lc3_os::traps::builtin::PUTS as Word;
        .FILL @NEW_PUTS;

        .ORIG #0x4000;
        @NEW_PUTS RTI;

        // We're going to access the memory mapped locations directly so we
        // don't want to be in user mode:
        .ORIG #lc3_os::USER_PROG_START_ADDR;
        .FILL #0x1000;

        // In protected space!
        .ORIG #0x1000;
        BRnzp @START;

        // See the note on the regular program for details about the below.
        @C_A .FILL #(((num_elements >> 48) as Word).wrapping_add(1));
        @C_B .FILL #(((num_elements >> 32) as Word).wrapping_add(1));
        @C_C .FILL #(((num_elements >> 16) as Word).wrapping_add(1));
        @C_D .FILL #(((num_elements >> 00) as Word).wrapping_add(0));

        @KBDR_ADDR .FILL #(lc3_baseline_sim::KBDR_ADDR);
        @DDR_ADDR .FILL #(lc3_baseline_sim::DDR_ADDR);

        @START
        LD R5, @KBDR_ADDR;
        LD R7, @DDR_ADDR;

        LD R1, @C_A;
        LD R2, @C_B;
        LD R3, @C_C;
        LD R4, @C_D;

        BRnzp @D_LOOP;

        @A_LOOP BRz @A_END;
            @B_LOOP BRz @B_END;
                @C_LOOP BRz @C_END;
                    @D_LOOP BRz @D_END;
                        LDR R0, R5, #0;
                        STR R0, R7, #0;

                        ADD R4, R4, #-1;
                        BRnzp @D_LOOP;
                    @D_END
                        ADD R4, R4, #-1;
                        ADD R3, R3, #-1;
                        BRnzp @C_LOOP;
                @C_END
                    ADD R3, R3, #-1;
                    ADD R2, R2, #-1;
                    BRnzp @B_LOOP;
            @B_END
                ADD R2, R2, #-1;
                ADD R1, R1, #-1;
                BRnzp @A_LOOP;
        @A_END
            HALT;
    }.into();

    prog
}

const SIZES: &[u64] = &[1, 10, 100, 1000, 10_000, 50_000];

use criterion::{BatchSize, BenchmarkId, BenchmarkGroup, Bencher, Criterion, Throughput, PlotConfiguration, AxisScale};
use criterion::measurement::WallTime;
use lc3_baseline_sim::interp::MachineState;

use std::time::{Duration, Instant};

fn bench_io(c: &mut Criterion) {
    let flags = PeripheralInterruptFlags::new();
    let mut group = c.benchmark_group("i/o throughput");

    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    for size in SIZES.iter() {
        group.throughput(Throughput::Bytes(*size));

        let input_stream: Vec<u8> = byte_stream(*size as usize).collect();

        let program = program(*size);

        let image_traps = {
            let mut image = OS_IMAGE.clone();
            image.layer_loadable(&program);

            image
        };

        let image_raw = {
            let mut image = OS_IMAGE.clone();
            image.layer_loadable(&raw_io_program(*size));

            image
        };

        // LC3Tools runs take a while..
        group.sample_size(10);

        group.bench_with_input(
            BenchmarkId::new("LC3Tools", *size),
            size,
            |b, size| {
                let mut out = (0..*size).map(|_| 0).collect();

                b.iter_custom(|iters| {
                    let mut acc = Duration::new(0, 0);
                    for _ in 0..iters {
                        let mut sim = Lc3ToolsSim::new_with_buffers(&input_stream, &mut out);
                        sim.load_program(&program);

                        let start = Instant::now();
                        criterion::black_box(sim.run(0x3000).unwrap());
                        acc += start.elapsed();

                        drop(sim);
                        eq!(input_stream, out);
                    }

                    acc
                })
            }
        );

        group.sample_size(100);

        fn interp_bench(
            group: &mut BenchmarkGroup<WallTime>,
            name: impl Into<String>,
            size: &u64,
            flags: &PeripheralInterruptFlags,
            input_stream: &Vec<u8>,
            image: &MemoryDump,
        ) {
            group.bench_with_input(
                BenchmarkId::new(name, *size),
                size,
                |b, size| {
                    b.iter_custom(|iters| {
                        let mut acc = Duration::new(0, 0);

                        for _ in 0..iters {
                            let mut output = Vec::<u8>::with_capacity(*size as usize);
                            let borrow = Mutex::new(&mut output);
                            let mut int = interpreter(&image, &flags, input_stream.iter().copied(), borrow);

                            let start = Instant::now();
                            while let MachineState::Running = int.step() {}
                            acc += start.elapsed();

                            drop(int);
                            eq!(*input_stream, output);
                        }

                        acc
                    });
                }
            );
        }

        interp_bench(
            &mut group,
            "Bare Interpreter w/TRAPs",
            size,
            &flags,
            &input_stream,
            &image_traps,
        );

        interp_bench(
            &mut group,
            "Bare Interpreter w/raw IO",
            size,
            &flags,
            &input_stream,
            &image_raw,
        );

        // TODO: do one that uses interrupt I/O.
        // (imp, low-pri)
    }
}

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, bench_io);
criterion_main!(benches);
