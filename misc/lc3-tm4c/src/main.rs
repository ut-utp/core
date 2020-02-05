#![cfg_attr(not(test), no_std)]
// #![no_std]
#![no_main]

extern crate static_assertions as sa;

// pick a panicking behavior
// extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
extern crate tm4c123x_hal as hal;

mod memory;
mod peripherals;
// mod transport;
// mod util;

// use bytes::{Buf, BufMut};


use memory::Tm4cMemory;
use peripherals::{Tm4cGpio, Tm4cAdc, Tm4cPwm, Tm4cTimers, Tm4cClock, Tm4cInput, Tm4cOutput};

use core::fmt::Write;
use cortex_m_rt::entry;
use hal::prelude::*;
use cortex_m::asm;

use lc3_traits::control::rpc::SimpleEventFutureSharedState;

// static SHARED_STATE: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

use lc3_baseline_sim::interp::{Interpreter, InstructionInterpreter, InterpreterBuilder, PeripheralInterruptFlags};
use lc3_baseline_sim::sim::Simulator;

use lc3_traits::peripherals::PeripheralSet;
use lc3_traits::control::Control;

#[entry]
fn main() -> ! {
    type Interp<'a> = Interpreter<'a,
        Tm4cMemory,
        PeripheralSet<'a,
            Tm4cGpio<'a>,
            Tm4cAdc,
            Tm4cPwm,
            Tm4cTimers<'a>,
            Tm4cClock,
            Tm4cInput<'a>,
            Tm4cOutput<'a>,
        >
    >;

    let flags: PeripheralInterruptFlags = PeripheralInterruptFlags::new();
    let state: SimpleEventFutureSharedState = SimpleEventFutureSharedState::new();

    let mut interp: Interp = InterpreterBuilder::new()
        .with_defaults()
        .build();

    interp.reset();
    interp.init(&flags);

    let mut sim = Simulator::new_with_state(interp, &state);
    sim.reset();

    // loop {
    //     // your code goes here
    //     sim.step();
    // }

    let p = hal::Peripherals::take().unwrap();

    let mut sc = p.SYSCTL.constrain();
    sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
        hal::sysctl::CrystalFrequency::_16mhz,
        hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();

    let mut porta = p.GPIO_PORTA.split(&sc.power_control);
    let mut portb = p.GPIO_PORTB.split(&sc.power_control);

    let mut u0 = p.UART0;
    let mut u1 = p.UART1;

    // u0.lcrh.modify(|_, w| w.eps().bit(true));

    // Activate UART
    let mut uart = hal::serial::Serial::uart0(
        u0,
        porta
            .pa1
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        porta
            .pa0
            .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
        (),
        (),
        // 115200_u32.bps(),
        // 3_686_400_u32.bps(),
        // 460_800_u32.bps(),
        // 1_000_000_u32.bps(),
        // 921_600_u32.bps(),
        // 1_500_000_u32.bps(),
        1_500_000_u32.bps(),
        // 115200_u32.bps(),
        // 2_300_000_u32.bps(),
        // 2_300_000_u32.bps(),
        // 115200.bps(),
        // 200_000_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
        // hal::serial::NewlineMode::Binary,
        &clocks,
        &sc.power_control,
    );

    let mut bt = hal::serial::Serial::uart1(
        u1,
        portb
            .pb1
            .into_af_push_pull::<hal::gpio::AF1>(&mut portb.control),
        portb
            .pb0
            .into_af_push_pull::<hal::gpio::AF1>(&mut portb.control),
        (),
        (),
        // 115200_u32.bps(),
        // 3_686_400_u32.bps(),
        // 460_800_u32.bps(),
        // 1_000_000_u32.bps(),
        // 921_600_u32.bps(),
        // 1_500_000_u32.bps(),
        // 9600_u32.bps(),
        // 2_300_000_u32.bps(),
        // 2_300_000_u32.bps(),
        // 115200.bps(),
        // 200_000_u32.bps(),
        // 38400_u32.bps(),
        // 1_382_400_u32.bps(),
        // 115_200_u32.bps(),
        38_400_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
        // hal::serial::NewlineMode::Binary,
        &clocks,
        &sc.power_control,
    );

    let mut counter = 0u16;

    // for i in 0..core::u16::MAX {
    // for i in 0..0xFE00u16 {
    //     sim.write_word(i, counter as u16);
    //     // writeln!(uart, "Hello, world! counter={}", counter).unwrap();
    //     // writeln!(uart, "{:#?}", sim.step());
    //     writeln!(uart, "{:#04X} -> {:#04X}", i, sim.read_word(i));
    //     // sim.write_word(i, counter as u16);
    //     counter = counter.wrapping_add(1);
    // }

    // let mut buf: [u8; 1] = [0];
    // use core::io::Read;

    // loop {
    //     if let Ok(1) = uart.read(&mut buf) {
    //         write!(bt, buf[0]);
    //     }

    //     if let Ok(1) = bt.read(&mut buf) {
    //         write!(uart, buf[0]);
    //     }
    // }

    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");
    writeln!(uart, "HELLO!");

    // write!(bt, "AT\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");
    // write!(bt, "AT+NAME\r\n");

    // loop {
    //     if let Ok(w) = uart.read() { bt.write(w); }
    //     if let Ok(w) = bt.read() { uart.write(w); }
    // }

    // loop {
    //     writeln!(uart, "Hello, world! counter={}", counter).unwrap();
    //     writeln!(bt, "Hello, world! counter={}", counter).unwrap();
    //     // writeln!(uart, "{:#?}", sim.step());
    //     counter = counter.wrapping_add(1);
    // }

    for i in 0..0xFE00u16 {
        sim.write_word(i, counter as u16);
        writeln!(uart, "{:#04X} -> {:#04X}", i, sim.read_word(i));
        // writeln!(bt, "{:#04X} -> {:#04X}", i, sim.read_word(i));
        counter = counter.wrapping_add(1);
    }

    loop {
        // writeln!(uart, "Hello, world! counter={}", counter).unwrap();
        // writeln!(uart, "c={}", counter).unwrap();
        // if counter == 0 { writeln!(bt, "Hello, world! counter={}", counter).unwrap(); }
        // if counter % 10000 == 0 { writeln!(bt, "Hello, world! counter={}", counter).unwrap(); }
        // if counter % 20 == 0 { writeln!(bt, "c={}", counter).unwrap(); }
        writeln!(bt, "c={}", counter).unwrap();
        // writeln!(uart, "{:#?}", sim.step());
        counter = counter.wrapping_add(1);
    }
}

// #![no_std]
// #![no_main]

// extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate tm4c123x_hal as hal;

// use core::fmt::Write;
// use cortex_m_rt::entry;
// use hal::prelude::*;

// #[entry]
// fn main() -> ! {
//     let p = hal::Peripherals::take().unwrap();

//     let mut sc = p.SYSCTL.constrain();
//     sc.clock_setup.oscillator = hal::sysctl::Oscillator::Main(
//         hal::sysctl::CrystalFrequency::_16mhz,
//         hal::sysctl::SystemClock::UsePll(hal::sysctl::PllOutputFrequency::_80_00mhz),
//     );
//     let clocks = sc.clock_setup.freeze();

//     let mut porta = p.GPIO_PORTA.split(&sc.power_control);

//     // Activate UART
//     let mut uart = hal::serial::Serial::uart0(
//         p.UART0,
//         porta
//             .pa1
//             .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
//         porta
//             .pa0
//             .into_af_push_pull::<hal::gpio::AF1>(&mut porta.control),
//         (),
//         (),
//         115200_u32.bps(),
//         hal::serial::NewlineMode::SwapLFtoCRLF,
//         &clocks,
//         &sc.power_control,
//     );

//     let mut counter = 0u32;
//     loop {
//         writeln!(uart, "Hello, world! counter={}", counter).unwrap();
//         counter = counter.wrapping_add(1);
//     }
// }
