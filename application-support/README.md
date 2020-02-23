### `lc3-application-support` crate

[![](https://github.com/ut-utp/prototype/workflows/application-support/badge.svg)](https://github.com/ut-utp/prototype/actions?query=workflow%3Aapplication-support)

Supporting materials for devices _using_ the UTP LC-3 Simulator.

--

This contains a few things useful to things (applications) that communicate with simulators. Currently, this includes:
    * The `InputSink` and `OutputSource` traits: a way to abstract over `Input` and `Output` peripherals that allow the controller (i.e. the application) to provide the inputs/consume the outputs
    * Shim support:
        - A type (`Shims`) for applications to use when dealing with a simulator that uses the shims
        - And a constructor function for said type
