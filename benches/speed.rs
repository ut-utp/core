//! A benchmark that just tries to measure execution speed.

#![cfg_attr(feature = "nightly", feature(test))]
// #![feature(test)]
// TODO: have CI run this and give us reports

extern crate criterion;

#[path = "common.rs"]
mod common;
use common::*;

#[cfg(feature = "nightly")]
use std::hint::black_box;

// use criterion::black_box;
// ^ doesn't work so we just provide a shim for when we're not on nightly:
#[cfg(not(feature = "nightly"))]
fn black_box<T>(inp: T) -> T { inp }

// const ITERS: [Word; 6] = [1, 10, 100, 1000, 10_000, 50_000];
const ITERS: [Word; 5] = [1, 10, 100, 1000, 10_000];


use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, PlotConfiguration, AxisScale};
use lc3_baseline_sim::interp::MachineState;

fn bench_fib(c: &mut Criterion) {
    let flags = PeripheralInterruptFlags::new();
    let mut group = c.benchmark_group("execution speed: fib(24)");

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
                // eprintln!("hello!");
                // println!("hello!");
                let mut int = black_box(bare_interpreter(build_fib_memory_image(*num), &flags));
                b.iter(|| {
                    int.reset();
                    while let MachineState::Running = int.step() {}
                })
            },
        );
    }
}

fn bench_fib_alt() {
    let flags = PeripheralInterruptFlags::default();

    for num_iter in ITERS.iter() {
        let mut int = black_box(bare_interpreter(build_fib_memory_image(*num_iter), &flags));
        int.reset();

        while let MachineState::Running = int.step() {}
    }
}

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, bench_fib);
criterion_main!(benches);

// fn main() {
//     let mut crit = Default::default();

//     bench_fib(&mut crit);
//     // bench_fib_alt();
// }
