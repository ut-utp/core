//! A quick benchmark that measures how long it takes to execute a simple
//! program in a few configurations.
//!
//! This should tell us how much overhead each subsequent layer in the
//! configuration adds.

// TODO: have CI run this and give us reports

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

//// Benches ////

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, PlotConfiguration, AxisScale};
use lc3_baseline_sim::interp::MachineState;

// const ITERS: [Word; 10] = [1, 10, 100, 500, 1000, 5000, 10000, 25000, 50000, 65535];
const ITERS: [Word; 5] = [1, 10, 100, 500, 1000];
// 506, 1937, 16247, 79847, 159347, 795347,
// // 159 * x + 347

use lc3_traits::control::rpc::device::RW_CLONE;
use std::task::{Context, Waker, Poll};
use std::pin::Pin;
use std::future::Future;

fn bench_fib(c: &mut Criterion) {
    let flags = PeripheralInterruptFlags::new();
    let mut group = c.benchmark_group("fib(24)");

    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    group.plot_config(plot_config);

    for num_iter in ITERS.iter() {
        group.throughput(Throughput::Elements(fib_program_executed_insn_count(
            *num_iter,
        )));

        group.bench_with_input(
            BenchmarkId::new("LC3Tools", *num_iter),
            num_iter,
            |b, num| {
                let mut sim = Lc3ToolsSim::new();
                sim.load_program(&fib_program(*num));
                b.iter(|| {
                    sim.run(0x3000).unwrap();
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("Bare Interpreter - step", *num_iter),
            num_iter,
            |b, num| {
                let mut int = bare_interpreter(build_fib_memory_image(*num), &flags);
                b.iter(|| {
                    int.reset();
                    while let MachineState::Running = int.step() {}
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Simulator - step", *num_iter),
            num_iter,
            |b, num| {
                let mut sim = simulator(build_fib_memory_image(*num), &flags);
                b.iter(|| {
                    sim.reset();

                    // Since we didn't register any break or watch points, the only
                    // event that can happen is a halt. Until that happens, we'll keep
                    // running.
                    while let None = sim.step() {}
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Simulator - run_until_event", *num_iter),
            num_iter,
            |b, num| {
                let sim = simulator(build_fib_memory_image(*num), &FLAGS);
                let (chan, halt, next) = executor_thread(sim);

                b.iter(|| {
                    async_std::task::block_on(next(&chan));
                });

                halt(&chan);
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Remote Simulator - step: mpsc, transparent", *num_iter),
            num_iter,
            |b, num| {
                let (halt, mut sim) = remote_simulator(build_fib_memory_image(*num));
                b.iter(|| {
                    sim.reset();
                    while let None = sim.step() {}
                });

                halt.send(()).unwrap();
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Remote Simulator - run_until_event: mpsc, transparent", *num_iter),
            num_iter,
            |b, num| {
                let (halt_dev, sim) = remote_simulator(build_fib_memory_image(*num));
                let (chan, halt_exec, next) = executor_thread(sim);

                // let (halt_or_fut, rx_halt_or_fut) = channel();
                // let (tx_fut, rx_fut) = channel();
                // std::thread::spawn(move || {
                //     loop {
                //         match rx_halt_or_fut.try_recv() {
                //             Err(_) => sim.tick(),
                //             Ok(None) => break,
                //             Ok(Some(())) => {
                //                 sim.reset();
                //                 tx_fut.send(sim.run_until_event()).unwrap();
                //             }
                //         }
                //     }
                // });

                // let next = || { halt_or_fut.send(Some(())).unwrap(); rx_fut.recv().unwrap() };

                b.iter(|| {
                    async_std::task::block_on(next(&chan));
                    // println!("FIN!!\n")
                });

                // halt_or_fut.send(None).unwrap();
                halt_dev.send(()).unwrap();
                halt_exec(&chan);
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Remote Simulator - run_until_event [no separate thread]: mpsc, transparent", *num_iter),
            num_iter,
            |b, num| {
                let (halt_dev, mut sim) = remote_simulator(build_fib_memory_image(*num));

                b.iter(|| {
                    sim.reset();
                    let mut fut = sim.run_until_event();

                    loop {
                        if let Poll::Ready(_) = Pin::new(&mut fut).poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RW_CLONE(&())) } )) {
                            break;
                        }

                        sim.tick();
                    }
                });

                halt_dev.send(()).unwrap();
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Simulator - run_until_event [no separate thread]", *num_iter),
            num_iter,
            |b, num| {
                let mut sim = simulator(build_fib_memory_image(*num), &FLAGS);

                b.iter(|| {
                    sim.reset();
                    let mut fut = sim.run_until_event();

                    loop {
                        if let Poll::Ready(_) = Pin::new(&mut fut).poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RW_CLONE(&())) } )) {
                            break;
                        }

                        sim.tick();
                        sim.tick();
                    }
                });
            },
        );
    }
}

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, bench_fib);
criterion_main!(benches);
