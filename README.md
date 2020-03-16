## Undergraduate Teaching Platform: A Prototype 👷

[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fut-utp%2Fprototype%2Fbadge&style=for-the-badge)](https://github.com/ut-utp/prototype/actions) [![License: MPL-2.0](https://img.shields.io/github/license/ut-utp/prototype?color=orange&style=for-the-badge)](https://opensource.org/licenses/MPL-2.0)
--
[![](https://tokei.rs/b1/github/ut-utp/prototype)](https://github.com/ut-utp/prototype) [![codecov](https://codecov.io/gh/ut-utp/prototype/branch/master/graph/badge.svg)](https://codecov.io/gh/ut-utp/prototype)


🚧 🚧 This is very much not stable yet! 🚧 🚧

This repo houses the 'core' of the UTP platform which consists of these pieces:
 - Types and friends for the [LC-3](https://en.wikipedia.org/wiki/Little_Computer_3) ISA [as we know and love it](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf).
     + Lives in the [`lc3-isa` crate](isa/).
 - Traits defining the LC-3's [peripherals](traits/src/peripherals/), [memory](traits/src/memory.rs), and [control interface](traits/src/control.rs).
     + Lives in the [`lc3-traits` crate](traits/).
     + This is really the heart and soul of the platform.
 - An instruction level simulator for the LC-3.
     + Lives in the [`lc3-baseline-sim` crate](baseline-sim).
 - Example implementations of the LC-3 peripherals and memory suitable for simulating the peripherals on a computer.
     + Lives in the [`lc3-shims` crate](shims).
     + Unlike the other library crates, this one isn't `#![no_std]`!
 - Some helper proc-macros (in the [`lc3-macros` crate](macros)).
     + This also isn't `#![no_std]` but it doesn't really count.
 - A barebones OS that is virtually identical to [the one used in lc3tools](https://github.com/chiragsakhuja/lc3tools/blob/b5d7245aabc33a05f28cc124202fd1532b1d9609/backend/lc3os.cpp#L12-L673).
     + Lives in the [`lc3-os` crate](os).
 - Bits and bobs useful to _applications_ (things that interact with simulator implementations).
     + Lives in the [`lc3-application-support` crate](application-support).
     + [Currently has](application-support/README.md) a wrapper type for shims and an Input/Output peripheral abstraction for impls that are backed by a host.

<TODO: diagram>

At the moment, the primary 'users' of the platform are the following:
 - An implementation of the platform for the TI Launchpad.
     + Lives in the [`lc3-tm4c` crate](//github.com/ut-utp/tm4c).
 - A TUI that can interact with any UTP LC-3 simulator.
     + Lives in the [`lc3-tui` crate](//github.com/ut-utp/tui).
     + Works with instances of [the simulator](baseline-sim) as well as actual devices like the [TM4C](//github.com/ut-utp/tm4c).

TODO:
 - [ ] crate and doc badges on each crate
 - [ ] doc badge to gh pages on the top level README (this file)
 - CI:
    + [ ] release (on tag)
    + [ ] docs (upload to gh-pages)
    + [ ] coverage (still want codecov over coveralls but may acquiesce)
