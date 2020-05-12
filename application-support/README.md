### `lc3-application-support` crate

[![](https://github.com/ut-utp/prototype/workflows/application-support/badge.svg)](https://github.com/ut-utp/prototype/actions?query=workflow%3Aapplication-support)
[![Minimum supported Rust version](https://img.shields.io/badge/rustc-1.42+-red.svg?style=for-the-badge&logo=rust)](#minimum-supported-rust-version-msrv)

Supporting materials for devices _using_ the UTP LC-3 Simulator.

--

This contains a few things useful to things (applications) that communicate with simulators. Put another way, this crate contains the things that can be shared between the TUI and the GUI. Currently, this includes:
    * The `InputSink` and `OutputSource` traits: a way to abstract over `Input` and `Output` peripherals that allow the controller (i.e. the application) to provide the inputs/consume the outputs
    * Shim support:
        - A type (`Shims`) for applications to use when dealing with a simulator that uses the shims
        - And a constructor function for said type
    * Control impl + Input/Output + Shims initialization abstractions
        - The `Init` trait and the `BlackBox` type.
            + The [`init` module docs](src/init/mod.rs) have more details.
    * Event loop abstractions.
        - More info [here](src/event_loop.rs).

### Minimum Supported Rust Version (MSRV)

This crate is currently guaranteed to compile on stable Rust 1.42 and newer. We offer no guarantees that this will remain true in future releases but do promise to always support (at minimum) the latest stable Rust version and to document changes to the MSRV in the [changelog](CHANGELOG.md).
