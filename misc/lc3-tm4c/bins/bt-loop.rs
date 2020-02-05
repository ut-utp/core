#![cfg_attr(not(test), no_std)]
#![no_main]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
extern crate tm4c123x_hal as hal;

use core::fmt::Write;
use cortex_m_rt::entry;
use hal::prelude::*;
use cortex_m::asm;

#[entry]
fn main() -> ! {
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
        1_500_000_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
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
        // 1_382_400_u32.bps(),
        // 115_200_u32.bps(),
        38_400_u32.bps(),
        hal::serial::NewlineMode::SwapLFtoCRLF,
        &clocks,
        &sc.power_control,
    );

    let mut counter = 0u32;

    writeln!(uart, "HELLO!");
    write!(bt, "YO\r\n");

    loop {
        if let Ok(w) = uart.read() { bt.write(w); }
        if let Ok(w) = bt.read() { uart.write(w); }
    }
}
