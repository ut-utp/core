### `lc3-device-support` crate

[![](https://github.com/ut-utp/prototype/workflows/device-support/badge.svg)](https://github.com/ut-utp/prototype/actions?query=workflow%3Adevice-support)

Supporting materials for devices running the UTP LC-3 Simulator.

--

This contains macros and pieces that aid in implementing the peripheral traits and running the simulator on devices with [embedded-hal](https://docs.rs/embedded-hal/) support. This includes:
         * the uart transport layer
         * the `#![no_std]` compatible encoding layer (based on [`postcard`](https://github.com/jamesmunns/postcard))
         * (eventually (TODO)) the macros that, provided with `embedded-hal` compliant pins, provides you with peripheral trait impls
         * miscellaneous things necessary for the above like a simple FIFO

(TODO!)
