//! Trap vector numbers and documentation.
//!
//! # Quick Reference Table
//! | Vector #   | Name | Inputs | Outputs | Description |
//! |:----------:| :--- | :----- | :------ | :---------- |
//! | **`0x20`** | [GETC]             | none                                                                  | [`R0`] - character from keyboard   | Reads a character from the keyboard.                                           |
//! | **`0x21`** | [OUT]              | [`R0`] - character to write                                           | none                               | Writes a character to the console display.                                     |
//! | **`0x22`** | [PUTS]             | [`R0`] - address of first character                                   | none                               | Writes a string of ASCII characters to the console display.                    |
//! | **`0x23`** | [IN]               | none                                                                  | [`R0`] - character from keyboard   | Prints a prompt and reads a character from the keyboard.                       |
//! | **`0x24`** | [PUTSP]            | [`R0`] - address of first characters                                  | none                               | Writes a string of ASCII characters, stored compactly, to the console display. |
//! | **`0x25`** | [HALT]             | none                                                                  | none                               | Halt execution and print a message on the console.                             |
//! | **`0x30`** | [GPIO_INPUT]       | [`R0`] - [pin][gpin] #                                                | `n` bit                            | Puts a [GPIO] [pin][gpin] in [Input mode][gInput].                             |
//! | **`0x31`** | [GPIO_OUTPUT]      | [`R0`] - [pin][gpin] #                                                | `n` bit                            | Puts a [GPIO] [pin][gpin] in [Output mode][gOutput].                           |
//! | **`0x32`** | [GPIO_INTERRUPT]   | [`R0`] - [pin][gpin] # <br>[`R1`] - address of ISR                    | `n` bit                            | Puts a [GPIO] in [Interrupt mode][gInterrupt] and sets the ISR.                |
//! | **`0x33`** | [GPIO_DISABLED]    | [`R0`] - [pin][gpin] #                                                | `n` bit                            | Puts a [GPIO] [pin][gpin] in [Disabled mode][gDisabled].                       |
//! | **`0x34`** | [GPIO_GET_MODE]    | [`R0`] - [pin][gpin] #                                                | [`R0`] - [GPIO mode] <br>`n` bit   | Returns the [mode][gmode] of a [GPIO] [pin][gpin].                             |
//! | **`0x35`** | [GPIO_WRITE]       | [`R0`] - [pin][gpin] # <br>[`R1`] - data to write                     | `n` bit                            | Writes to a [GPIO] [pin][gpin] in [Output mode][gOutput].                      |
//! | **`0x36`** | [GPIO_READ]        | [`R0`] - [pin][gpin] #                                                | [`R0`] - data from pin <br>`n` bit | Reads data from a [GPIO] [pin][gpin].                                          |
//! | **`0x40`** | [ADC_ENABLE]       | [`R0`] - [pin][apin] #                                                | `n` bit                            | Puts an [ADC] [pin][apin] in [Enabled mode][aEnabled].                         |
//! | **`0x41`** | [ADC_DISABLE]      | [`R0`] - [pin][apin] #                                                | `n` bit                            | Puts an [ADC] [pin][apin] in [Disabled mode][aDisabled].                       |
//! | **`0x42`** | [ADC_GET_MODE]     | [`R0`] - [pin][apin] #                                                | [`R0`] - [ADC mode] <br>`n` bit    | Returns the mode of an [ADC] [pin][apin].                                      |
//! | **`0x43`** | [ADC_READ]         | [`R0`] - [pin][apin] #                                                | [`R0`] - data from pin <br>`n` bit | Reads data from an [ADC] [pin][apin].                                          |
//! | **`0x50`** | [PWM_ENABLE]       | [`R0`] - [pin][ppin] # <br>[`R1`] - period <br>[`R2`] - duty cycle    | `n` bit                            | Puts a [PWM] in [Enabled mode][pEnabled], with period and duty cycle.          |
//! | **`0x51`** | [PWM_DISABLE]      | [`R0`] - [pin][ppin] #                                                | `n` bit                            | Puts a [PWM] [pin][ppin] in [Disabled mode][pDisabled].                        |
//! | **`0x52`** | [PWM_GET_PERIOD]   | [`R0`] - [pin][ppin] #                                                | [`R0`] - period <br>`n` bit        | Returns the period of a [PWM pin][ppin].                                       |
//! | **`0x53`** | [PWM_GET_DUTY]     | [`R0`] - [pin][ppin] #                                                | [`R0`] - duty cycle <br>`n` bit    | Returns the duty cycle of a [PWM pin][ppin].                                   |
//! | **`0x60`** | [TIMER_SINGLESHOT] | [`R0`] - [id][tid] # <br>[`R1`] - period <br>[`R2`] - address of ISR  | `n` bit                            | Puts a [Timer] in [SingleShot mode][tSingleShot] with period and sets the ISR. |
//! | **`0x61`** | [TIMER_REPEATED]   | [`R0`] - [id][tid] # <br>[`R1`] - period <br>[`R2`] - address of ISR  | `n` bit                            | Puts a [Timer] in [Repeated mode][tRepeated] with period and sets the ISR.     |
//! | **`0x62`** | [TIMER_DISABLE]    | [`R0`] - [id][tid] #                                                  | `n` bit                            | Puts a [Timer] in [Disabled mode][tDisabled].                                  |
//! | **`0x63`** | [TIMER_GET_MODE]   | [`R0`] - [id][tid] #                                                  | [`R0`] - [Timer mode] <br>`n` bit  | Returns the [mode][tMode] of a [Timer].                                        |
//! | **`0x64`** | [TIMER_GET_PERIOD] | [`R0`] - [id][tid] #                                                  | [`R0`] - period                    | Returns the [period][tState] of a [Timer].                                     |
//! | **`0x70`** | [CLOCK_SET]        | [`R0`] - value to set                                                 | none                               | Sets the value of the [Clock].                                                 |
//! | **`0x71`** | [CLOCK_GET]        | none                                                                  | [`R0`] - value of clock            | Gets the value of the [Clock].                                                 |
//!
//! [GETC]: builtin::GETC
//! [OUT]: builtin::OUT
//! [PUTS]: builtin::PUTS
//! [IN]: builtin::IN
//! [PUTSP]: builtin::PUTSP
//! [HALT]: builtin::HALT
//!
//! [GPIO_INPUT]: gpio::INPUT
//! [GPIO_OUTPUT]: gpio::OUTPUT
//! [GPIO_INTERRUPT]: gpio::INTERRUPT
//! [GPIO_DISABLED]: gpio::DISABLED
//! [GPIO_GET_MODE]: gpio::GET_MODE
//! [GPIO_WRITE]: gpio::WRITE
//! [GPIO_READ]: gpio::READ
//! [ADC_ENABLE]: adc::ENABLE
//! [ADC_DISABLE]: adc::DISABLE
//! [ADC_GET_MODE]: adc::GET_MODE
//! [ADC_READ]: adc::READ
//! [PWM_ENABLE]: pwm::ENABLE
//! [PWM_DISABLE]: pwm::DISABLE
//! [PWM_GET_PERIOD]: pwm::GET_PERIOD
//! [PWM_GET_DUTY]: pwm::GET_DUTY
//! [TIMER_SINGLESHOT]: timers::SINGLESHOT
//! [TIMER_REPEATED]: timers::REPEATED
//! [TIMER_DISABLE]: timers::DISABLE
//! [TIMER_GET_MODE]: timers::GET_MODE
//! [TIMER_GET_PERIOD]: timers::GET_PERIOD
//! [CLOCK_SET]: clock::SET
//! [CLOCK_GET]: clock::GET
//!
//! [`R0`]: lc3_isa::Reg::R0
//! [`R1`]: lc3_isa::Reg::R1
//! [`R2`]: lc3_isa::Reg::R2
//!
//! [GPIO]: lc3_traits::peripherals::gpio::Gpio
//! [gpin]: lc3_traits::peripherals::gpio::GpioPin
//! [gmode]: lc3_traits::peripherals::gpio::GpioState
//! [gInput]: lc3_traits::peripherals::gpio::GpioState::Input
//! [gOutput]: lc3_traits::peripherals::gpio::GpioState::Output
//! [gInterrupt]: lc3_traits::peripherals::gpio::GpioState::Interrupt
//! [gDisabled]: lc3_traits::peripherals::gpio::GpioState::Disabled
//!
//! [ADC]: lc3_traits::peripherals::adc::Adc
//! [apin]: lc3_traits::peripherals::adc::AdcPin
//! [aEnabled]: lc3_traits::peripherals::adc::AdcState::Enabled
//! [aDisabled]: lc3_traits::peripherals::adc::AdcState::Disabled
//!
//! [PWM]: lc3_traits::peripherals::pwm::Pwm
//! [ppin]: lc3_traits::peripherals::pwm::PwmPin
//! [pEnabled]: lc3_traits::peripherals::pwm::PwmState::Enabled
//! [pDisabled]: lc3_traits::peripherals::pwm::PwmState::Disabled
//!
//! [Timer]: lc3_traits::peripherals::timers::Timers
//! [tid]: lc3_traits::peripherals::timers::TimerId
//! [tSingleShot]: lc3_traits::peripherals::timers::TimerMode::SingleShot
//! [tRepeated]: lc3_traits::peripherals::timers::TimerMode::Repeated
//! [tDisabled]: lc3_traits::peripherals::timers::TimerState::Disabled
//! [tMode]: lc3_traits::peripherals::timers::TimerMode
//! [tState]: lc3_traits::peripherals::timers::TimerState
//!
//! [Clock]: lc3_traits::peripherals::clock::Clock
//!
//! [GPIO Mode]: lc3_traits::peripherals::gpio::GpioState
//! [ADC Mode]: lc3_traits::peripherals::adc::AdcState
//! [Timer Mode]: lc3_traits::peripherals::timers::TimerMode
//!
//! # Guidelines on Writing ISRs
//!
//! Properly written ISRs should:
//!   - Save and restore all registers they modify _using the stack_.
//!      * It's important to use the stack to do this instead of using special
//!        save locations in memory for _reentrancy_.
//!      * If your ISR is guaranteed to only going to run once (i.e. it's only
//!        registered as the handler for one interrupt) **and** the save
//!        locations you're using aren't shared with any other code, then you
//!        can get away with using special save locations.
//!          + Even in this situation, we recommend you use the stack since
//!            doing so consolidates all the regions of memory that are
//!            modified; this means the underlying memory implementation will
//!            have to keep track of fewer modified pages and won't need to
//!            spend as much time paging.
//!  - Make that when you return from your ISR, the stack pointer is the same as
//!    when your ISR began running.
//!     * Put another way: Have an equal number of pushes and pops.
//!  - When pushing and popping, make sure you decrement the stack pointer
//!    ([`R6`]) **before** pushing and increment it **after** popping.
//!     * This is subtle, but if you were to go the other way (i.e. decrement
//!       the stack pointer after pushing registers) you run the risk of losing
//!       your data (if an interrupt with a higher priority were to come along
//!       and push things, pop them, and return). The same applies to popping
//!       after you increment the stack pointer.
//        + TODO: example
//!     * If you follow this rule you can push/pop multiple things onto/off of
//!       the stack in one go by decrementing/incrementing the stack pointer by
//!       more than 1.
//!  - The OS, by default,
//!    [provisions about a page](crate::OS_DEFAULT_STARTING_SP) (256 memory
//!    locations) of stack space and sets the starting stack pointer
//!    accordingly.
//!     * If you overrun this, you'll start to write over the OS!
//!  - Return from your ISRs with an [`RTI`]!
//!     * Note that this means you cannot call your ISRs directly from user
//!       code (i.e. `JSR ISR` won't work).
//!
//! [`R6`]: lc3_isa::Reg::R6
//! [`RTI`]: lc3_isa::Instruction::Rti


use lc3_baseline_sim::mem_mapped as mm;

/// Uses the workaround detailed
/// [here](https://github.com/rust-lang/rust/issues/52607) to let us 'generate'
/// a doc literal.
macro_rules! calculated_doc {
  ( $thing:item $(#[doc = $doc:expr])* ) => {
      $(
          #[doc = $doc]
      )*
          $thing
  };
}

macro_rules! define {
  ([$starting:expr] <- { $(#[doc = $doc:expr])* $([$chk:literal])? $first:ident $(,)? $( $(#[doc = $r_doc:expr])* $([$r_chk:literal])? $rest:ident$(,)?)* }) => {
      calculated_doc! {
          pub const $first: u8 = $starting;
          $(#[doc = concat!("**`[", stringify!($chk), "]`** ")])*
          $(#[doc = $doc])*
          $(#[doc = concat!("\n ### TRAP Vector\n Vector Number: **`", stringify!($chk), "`**")])?
      }

      $(sa::const_assert_eq!($first, $chk);)?

      define!(munch $first $( $(#[doc = $r_doc])* $([$r_chk])? $rest )* );
  };

  (munch $previous:ident $(#[doc = $doc:expr])* $([$chk:literal])? $next:ident $( $(#[doc = $r_doc:expr])* $([$r_chk:literal])? $rest:ident)*) => {
      calculated_doc! {
          pub const $next: u8 = $previous + 1;
          $(#[doc = concat!("**`[", stringify!($chk), "]`** ")])*
          $(#[doc = $doc])*
          $(#[doc = concat!("\n ### TRAP Vector\n Vector Number: **`", stringify!($chk), "`**")])?
      }

      $(sa::const_assert_eq!($next, $chk);)?

      define!(munch $next $( $(#[doc = $r_doc])* $([$r_chk])? $rest )* );
  };

  (munch $previous:ident) => { }
}

// Note: we're currently using ARM Assembly for syntax highlighting but this
// is not perfect for us.

/// Trap vectors for the [`Gpio`](lc3_traits::peripherals::Gpio) peripheral.
pub mod gpio {
  define!([super::mm::GPIO_OFFSET] <- {
      /// Puts a [GPIO] [Pin] in [Input] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A GPIO [pin] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [GPIO] [Pin] indicated by [`R0`] into [Input]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Input]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x30           ; Sets G0 to Input
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [Input]: lc3_traits::peripherals::gpio::GpioState::Input
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x30] INPUT,
      /// Puts a [GPIO] [Pin] in [Output] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [GPIO] [Pin] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [GPIO] [Pin] indicated by [`R0`] into [Output]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x31           ; Sets G0 to Output
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [Output]: lc3_traits::peripherals::gpio::GpioState::Output
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x31] OUTPUT,
      /// Puts a [GPIO] [Pin] in [Interrupt] mode and sets the interrupt service
      /// routine address in the interrupt vector table.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [GPIO] [Pin] number.
      ///  - [`R1`]: Address of interrupt service routine.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [GPIO] [Pin] indicated by [`R0`] into [Interrupt]
      /// mode. This TRAP also sets the corresponding interrupt vector table
      /// entry for this [GPIO] [Pin] to the address indicated by [`R1`].
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`] and [`R1`]) are preserved.
      ///
      /// Be sure to follow the
      /// [guidelines for writing ISRs](../index.html#guidelines-on-writing-isrs).
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Interrupt] and sets the interrupt
      /// service routine to `ISR`. It will then spin until [`G0`] fires an
      /// interrupt, which will call `ISR`, set the `ISR_FLAG` to 1, and allow
      /// the main program to halt.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LEA R1, ISR         ; Sets R1 to the address of ISR
      /// TRAP 0x32           ; Sets G0 to Interrupt w/ ISR
      ///
      /// LOOP
      /// LD R1, ISR_FLAG     ; Loops until the flag is set
      /// BRz LOOP
      /// HALT
      ///
      /// ISR_FLAG .FILL #0
      ///
      /// ISR
      /// ADD R6, R6, #-1     ; Save R0 onto the supervisor stack
      /// STR R0, R6, #0
      ///
      /// AND R0, R0, #0      ; Sets the flag
      /// ADD R0, R0, #1
      /// ST R0, ISR_FLAG
      ///
      /// LDR R0, R6, #0      ; Pop R0 off of the supervisor stack
      /// ADD R6, R6, #1
      /// RTI
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [Interrupt]: lc3_traits::peripherals::gpio::GpioState::Interrupt
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x32] INTERRUPT,
      /// Puts a [GPIO] [Pin] in [Disabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [GPIO] [Pin] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [GPIO] [Pin] indicated by [`R0`] into [Disabled]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output], then immediately sets it
      /// to [Disabled]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x31           ; Sets G0 to Output
      /// TRAP 0x33           ; Sets G0 to Disabled
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [Disabled]: lc3_traits::peripherals::gpio::GpioState::Disabled
      /// [Output]: lc3_traits::peripherals::gpio::GpioState::Output
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x33] DISABLED,
      /// Returns the mode of a [GPIO] [Pin].
      ///
      /// ## Inputs
      ///  - [`R0`]: A [GPIO] [Pin] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: A value corresponding to a [GPIO] [mode].
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP returns the mode of the [GPIO] [Pin] indicated by [`R0`] by
      /// writing a value to [`R0`]. The values are as follows:
      ///
      /// | Mode          | Value |
      /// | ------------- | ----- |
      /// | [`Disabled`]  | 0     |
      /// | [`Output`]    | 1     |
      /// | [`Input`]     | 2     |
      /// | [`Interrupt`] | 3     |
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output], then reads [`G0`]'s mode
      /// into [`R0`]. [`R0`] will then contain the value 1.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x31           ; Sets G0 to Output
      /// TRAP 0x34           ; Reads G0's mode, sets R0 to 1
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [mode]: lc3_traits::peripherals::gpio::GpioState
      /// [`Input`]: lc3_traits::peripherals::gpio::GpioState::Input
      /// [`Output`]: lc3_traits::peripherals::gpio::GpioState::Output
      /// [`Interrupt`]: lc3_traits::peripherals::gpio::GpioState::Interrupt
      /// [`Disabled`]: lc3_traits::peripherals::gpio::GpioState::Disabled
      /// [Output]: lc3_traits::peripherals::gpio::GpioState::Output
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x34] GET_MODE,
      /// Writes data to a [GPIO] [Pin] in [Output] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [GPIO] [Pin] number.
      ///  - [`R1`]: Data to write.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP writes the data in [`R1`] to the [GPIO] [Pin] indicated
      /// by [`R0`].
      ///
      /// Only the least significant bit of [`R1`] is written.
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// Attempting to write to a [GPIO] [Pin] that is not in [Output] mode
      /// does nothing.
      ///
      /// All registers (including [`R0`] and [`R1`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output], then writes the value 1 to
      /// [`G0`]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x31           ; Sets G0 to Output
      /// AND R1, R1, #0      ; Sets R1 to 1
      /// ADD R1, R1, #1
      /// TRAP 0x35           ; Writes 1 to G0
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [Output]: lc3_traits::peripherals::gpio::GpioState::Output
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x35] WRITE,
      /// Reads data from a [GPIO] [Pin] in [Input] or [Interrupt] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [GPIO] [Pin] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: data from [GPIO] [Pin]
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP reads data from the [GPIO] [Pin] indicated by [`R0`], and
      /// returns the data in [`R0`].
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// Attempting to read from a [GPIO] [Pin] that is not in [Input] or
      /// [Interrupt] mode returns -1 in [`R0`].
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Input], then reads from [`G0`] into
      /// [`R0`]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x30           ; Sets G0 to Input
      /// TRAP 0x36           ; Reads from G0, sets R0
      /// ```
      ///
      /// [GPIO]: lc3_traits::peripherals::Gpio
      /// [Input]: lc3_traits::peripherals::gpio::GpioState::Input
      /// [Interrupt]: lc3_traits::peripherals::gpio::GpioState::Interrupt
      /// [Pin]: lc3_traits::peripherals::gpio::GpioPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_GPIO_PINS`]: lc3_traits::peripherals::gpio::GpioPin::NUM_PINS
      /// [`G0`]: lc3_traits::peripherals::gpio::GpioPin::G0
      [0x36] READ,
  });
}

/// Trap vectors for the [`Adc`](lc3_traits::peripherals::Adc) peripheral.
pub mod adc {
  define!([super::mm::ADC_OFFSET] <- {
      /// Puts an [ADC] [Pin] in [Enabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: An [ADC] [Pin] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [ADC] [Pin] indicated by [`R0`] into [Enabled]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_ADC_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`A0`] to be an [Enabled]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x40           ; Sets A0 to Enabled
      /// ```
      ///
      /// [ADC]: lc3_traits::peripherals::adc
      /// [Enabled]: lc3_traits::peripherals::adc::AdcState::Enabled
      /// [Pin]: lc3_traits::peripherals::adc::AdcPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_ADC_PINS`]: lc3_traits::peripherals::adc::AdcPin::NUM_PINS
      /// [`A0`]: lc3_traits::peripherals::adc::AdcPin::A0
      [0x40] ENABLE,
      /// Puts an [ADC] [Pin] in [Disabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: An [ADC] [Pin] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [ADC] [Pin] indicated by [`R0`] into [Disabled]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_ADC_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`A0`] to be an [Enabled], then immediately sets it
      /// to [Disabled]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x40           ; Sets A0 to Enabled
      /// TRAP 0x41           ; Sets A0 to Disabled
      /// ```
      ///
      /// [ADC]: lc3_traits::peripherals::adc
      /// [Enabled]: lc3_traits::peripherals::adc::AdcState::Enabled
      /// [Disabled]: lc3_traits::peripherals::adc::AdcState::Disabled
      /// [Pin]: lc3_traits::peripherals::adc::AdcPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_ADC_PINS`]: lc3_traits::peripherals::adc::AdcPin::NUM_PINS
      /// [`A0`]: lc3_traits::peripherals::adc::AdcPin::A0
      [0x41] DISABLE,
      /// Returns the mode of an [ADC] [Pin].
      ///
      /// ## Inputs
      ///  - [`R0`]: An [ADC] [Pin] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: A value corresponding to an [ADC] [mode].
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP returns the mode of the [ADC] [Pin] indicated by [`R0`] by
      /// writing a value to [`R0`]. The values are as follows:
      ///
      /// | Mode          | Value |
      /// | ------------- | ----- |
      /// | [`Disabled`]  | 0     |
      /// | [`Enabled`]   | 1     |
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_ADC_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`A0`] to be an [Disabled], then reads [`A0`]'s mode
      /// into [`R0`]. [`R0`] will then contain the value 1.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x41           ; Sets A0 to Disabled
      /// TRAP 0x42           ; Reads A0's mode, sets R0 to 1
      /// ```
      ///
      /// [ADC]: lc3_traits::peripherals::adc
      /// [mode]: lc3_traits::peripherals::adc::AdcState
      /// [`Enabled`]: lc3_traits::peripherals::adc::AdcState::Enabled
      /// [`Disabled`]: lc3_traits::peripherals::adc::AdcState::Disabled
      /// [Disabled]: lc3_traits::peripherals::adc::AdcState::Disabled
      /// [Pin]: lc3_traits::peripherals::adc::AdcPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_ADC_PINS`]: lc3_traits::peripherals::adc::AdcPin::NUM_PINS
      /// [`A0`]: lc3_traits::peripherals::adc::AdcPin::A0
      [0x42] GET_MODE,
      /// Reads data from an [ADC] [Pin] in [Enabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: An [ADC] [Pin] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: data from [ADC] [Pin]
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP reads data from the [ADC] [Pin] indicated by [`R0`], and
      /// returns the data in [`R0`].
      ///
      /// The data returned will be in the range \[0, 255\].
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_ADC_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// Attempting to read from an [ADC] [Pin] that is not in [Enabled]
      /// mode returns -1 in [`R0`].
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`A0`] to be an [Enabled], then reads from [`A0`] into
      /// [`R0`]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x40           ; Sets A0 to Enabled
      /// TRAP 0x43           ; Reads from A0, sets R0
      /// ```
      ///
      /// [ADC]: lc3_traits::peripherals::adc
      /// [mode]: lc3_traits::peripherals::adc::AdcState
      /// [Enabled]: lc3_traits::peripherals::adc::AdcState::Enabled
      /// [Pin]: lc3_traits::peripherals::adc::AdcPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_ADC_PINS`]: lc3_traits::peripherals::adc::AdcPin::NUM_PINS
      /// [`A0`]: lc3_traits::peripherals::adc::AdcPin::A0
      [0x43] READ,
  });
}

/// Trap vectors for the [`Pwm`](lc3_traits::peripherals::Pwm) peripheral.
pub mod pwm {
  define!([super::mm::PWM_OFFSET] <- {
      /// Puts a [PWM] [Pin] in [Enabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [PWM] [Pin] number.
      ///  - [`R1`]: The period.
      ///  - [`R2`]: The duty cycle.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [PWM] [Pin] indicated by [`R0`] into [Enabled]
      /// mode. It also sets the corresponding period and duty cycle of that
      /// [Pin] with [`R1`] and [`R2`] respectively.
      ///
      /// The period and duty cycle will only use the 8 least significant bits
      /// of [`R1`] and [`R2`], resulting in values in the range \[0, 255\].
      /// The period is measured in units of milliseconds. The duty cycle is
      /// the fractional value (e.g. a value of 64 corresponds to a 25% duty
      /// cycle).
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_PWM_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`], [`R1`], and [`R2`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and
      /// a *50%* duty cycle then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 20
      /// LD R2, DUTY         ; Sets R2 to 128
      /// TRAP 0x50           ; Sets P0 to enabled w/ period of 20 ms and duty
      ///                     ; of 50%
      /// HALT
      ///
      /// PERIOD .FILL #20
      /// DUTY .FILL #128
      /// ```
      ///
      /// [PWM]: lc3_traits::peripherals::pwm
      /// [Enabled]: lc3_traits::peripherals::pwm::PwmState::Enabled
      /// [Pin]: lc3_traits::peripherals::pwm::PwmPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_PWM_PINS`]: lc3_traits::peripherals::pwm::PwmPin::NUM_PINS
      /// [`P0`]: lc3_traits::peripherals::pwm::PwmPin::P0
      [0x50] ENABLE,
      /// Puts a [PWM] [Pin] in [Disabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [PWM] [Pin] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [PWM] [Pin] indicated by [`R0`] into [Disabled]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_PWM_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and
      /// a *50%* duty cycle, immediately sets it to [Disabled], then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 20
      /// LD R2, DUTY         ; Sets R2 to 128
      /// TRAP 0x50           ; Sets P0 to Enabled w/ period of 20 ms and duty
      ///                     ; of 50%
      /// TRAP 0x51           ; Sets P0 to Disabled
      /// HALT
      ///
      /// PERIOD .FILL #20
      /// DUTY .FILL #128
      /// ```
      ///
      /// [PWM]: lc3_traits::peripherals::pwm
      /// [Enabled]: lc3_traits::peripherals::pwm::PwmState::Enabled
      /// [Disabled]: lc3_traits::peripherals::pwm::PwmState::Disabled
      /// [Pin]: lc3_traits::peripherals::pwm::PwmPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_PWM_PINS`]: lc3_traits::peripherals::pwm::PwmPin::NUM_PINS
      /// [`P0`]: lc3_traits::peripherals::pwm::PwmPin::P0
      [0x51] DISABLE,
      /// Reads the period of a [PWM] [Pin] in [Enabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [PWM] [Pin] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: A period ∈ \[0, 255\] milliseconds.
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP reads the period from the [PWM] [Pin] indicated by [`R0`]
      /// and returns the period in [`R0`]. The period will be a value in the
      /// range \[0, 255\] and has units of milliseconds.
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_PWM_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// Attempting to read the period from a [PWM] [Pin] that is in [Disabled]
      /// mode returns 0 in [`R0`].
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and
      /// a *50%* duty cycle. It then reads the period of [`P0`] and results in
      /// the value 20 in [`R0`] then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 20
      /// LD R2, DUTY         ; Sets R2 to 128
      /// TRAP 0x50           ; Sets P0 to Enabled w/ period of 20 ms and duty
      ///                     ; of 50%
      /// TRAP 0x52           ; Reads period of P0, sets R0 to 20
      /// HALT
      ///
      /// PERIOD .FILL #20
      /// DUTY .FILL #128
      /// ```
      ///
      /// [PWM]: lc3_traits::peripherals::pwm
      /// [Enabled]: lc3_traits::peripherals::pwm::PwmState::Enabled
      /// [Disabled]: lc3_traits::peripherals::pwm::PwmState::Disabled
      /// [Pin]: lc3_traits::peripherals::pwm::PwmPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_PWM_PINS`]: lc3_traits::peripherals::pwm::PwmPin::NUM_PINS
      /// [`P0`]: lc3_traits::peripherals::pwm::PwmPin::P0
      [0x52] GET_PERIOD,
      /// Reads the duty cycle of a [PWM] [Pin] in [Enabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [PWM] [Pin] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: A duty cycle ∈ \[0, 255\].
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP reads the duty cycle from the [PWM] [Pin] indicated by
      /// [`R0`] and returns the duty cycle in [`R0`]. The duty cycle will
      /// be a value in the range \[0, 255\] and corresponds to a percentage
      /// (e.g. a value of 64 corresponds to a 25% duty cycle).
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_PWM_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// Attempting to read the duty cycle from a [PWM] [Pin] that is in
      /// [Disabled] mode returns -1 in [`R0`].
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and
      /// a *50%* duty cycle. It then reads the duty cycle of [`P0`] and results
      /// in the value 128 in [`R0`] then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 20
      /// LD R2, DUTY         ; Sets R2 to 128
      /// TRAP 0x50           ; Sets P0 to Enabled w/ period of 20 ms and duty
      ///                     ; of 50%
      /// TRAP 0x53           ; Reads duty of P0, sets R0 to 128
      /// HALT
      ///
      /// PERIOD .FILL #20
      /// DUTY .FILL #128
      /// ```
      ///
      /// [PWM]: lc3_traits::peripherals::pwm
      /// [Enabled]: lc3_traits::peripherals::pwm::PwmState::Enabled
      /// [Disabled]: lc3_traits::peripherals::pwm::PwmState::Disabled
      /// [Pin]: lc3_traits::peripherals::pwm::PwmPin
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`NUM_PWM_PINS`]: lc3_traits::peripherals::pwm::PwmPin::NUM_PINS
      /// [`P0`]: lc3_traits::peripherals::pwm::PwmPin::P0
      [0x53] GET_DUTY,
  });
}

/// Trap vectors for the [`Timers`](lc3_traits::peripherals::Timers)
/// peripheral.
pub mod timers {
  define!([super::mm::TIMER_OFFSET] <- {
      /// Puts a [Timer] in [SingleShot] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [Timer] [ID] number.
      ///  - [`R1`]: The period.
      ///  - [`R2`]: Address of interrupt service routine.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [Timer] indicated by [`R0`] into [SingleShot]
      /// mode. It also sets the period of the [Timer] as well as the entry
      /// in the interrupt vector table corresponding to the [Timer]. The
      /// period is measured in units of milliseconds and uses full 16-bit
      /// words (i.e. the period will be a value in the range \[0, 65535\]).
      ///
      /// In [SingleShot] mode, the interrupt service routine is only
      /// triggered one time. The [Timer] will then set its state to [Disabled].
      ///
      /// Setting the mode or period of a [Timer] will overwrite the previous
      /// [Timer]. In other words, if this TRAP is called on a [Timer] that is
      /// currently running, the interrupt service routine will only trigger
      /// after the newly set period elapses.
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid timer [ID] number, the `n` bit is
      /// set.
      ///
      /// All registers (including [`R0`], [`R1`], and [`R2`]) are preserved.
      ///
      /// Be sure to follow the
      /// [guidelines for writing ISRs](../index.html#guidelines-on-writing-isrs).
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [SingleShot] with a period of `3
      /// seconds` and sets the interrupt service routine to `ISR`. It will then
      /// spin until [`T0`] fires an interrupt, after three seconds have passed,
      /// which will call `ISR`, set the `ISR_FLAG` to 1, and allow the main
      /// program to halt.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 3000
      /// LEA R2, ISR         ; Sets R2 to the address of ISR
      /// TRAP 0x60           ; Sets T0 to SingleShot w/ period of 3000 and ISR
      ///
      /// LOOP
      /// LD R1, ISR_FLAG     ; Loops until flag is set
      /// BRz LOOP
      /// HALT
      ///
      /// PERIOD .FILL #3000
      /// ISR_FLAG .FILL #0
      ///
      /// ISR
      /// ADD R6, R6, #-1     ; Save R0 onto the supervisor stack
      /// STR R0, R6, #0
      ///
      /// AND R0, R0, #0
      /// ADD R0, R0, #1
      /// ST R0, ISR_FLAG
      ///
      /// LDR R0, R6, #0      ; Pop R0 off of the supervisor stack
      /// ADD R6, R6, #1
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [Disabled]: lc3_traits::peripherals::timers::TimerState::Disabled
      /// [SingleShot]: lc3_traits::peripherals::timers::TimerMode::SingleShot
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
      [0x60] SINGLESHOT,
      /// Puts a [Timer] in [Repeated] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [Timer] [ID] number.
      ///  - [`R1`]: The period.
      ///  - [`R2`]: Address of interrupt service routine.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP puts the [Timer] indicated by [`R0`] into [Repeated]
      /// mode. It also sets the period of the [Timer] as well as the entry
      /// in the interrupt vector table corresponding to the [Timer]. The
      /// period is measured in units of milliseconds and uses full 16-bit
      /// words (i.e. the period will be a value in the range \[0, 65535\]).
      ///
      /// In [Repeated] mode, the interrupt service routine triggers every
      /// period cycle until the [Timer] is disabled.
      ///
      /// Setting the mode or period of a [Timer] will overwrite the previous
      /// [Timer]. In other words, if this TRAP is called on a [Timer] that is
      /// currently running, the interrupt service routine will only trigger
      /// after the newly set period elapses.
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid timer [ID] number, the `n` bit is
      /// set.
      ///
      /// All registers (including [`R0`], [`R1`], and [`R2`]) are preserved.
      ///
      /// Be sure to follow the
      /// [guidelines for writing ISRs](../index.html#guidelines-on-writing-isrs).
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [Repeated] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. When [`T0`] fires an
      /// interrupt every second, `ISR` is called, which increments a counter.
      /// In the main program, the counter is checked repeatedly until the
      /// target of `10` is reached.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 1000
      /// LEA R2, ISR         ; Sets R2 to the address of ISR
      /// TRAP 0x61           ; Sets T0 to Repeated w/ period of 1000 and ISR
      ///
      /// LD R1, TARGET       ; Sets R1 to -10
      /// LOOP
      /// LD R0, COUNTER      ; Loops until counter reaches 10
      /// ADD R0, R0, R1
      /// BRn LOOP
      /// HALT
      ///
      /// PERIOD .FILL #1000
      /// COUNTER .FILL #0
      /// TARGET .FILL #-10
      ///
      /// ISR
      /// ADD R6, R6, #-1     ; Save R0 onto the supervisor stack
      /// STR R0, R6, #0
      ///
      /// LD R0, COUNTER      ; Increment the counter
      /// ADD R0, R0, #1
      /// ST R0, COUNTER
      ///
      /// LDR R0, R6, #0      ; Pop R0 off of the supervisor stack
      /// ADD R6, R6, #1
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [Repeated]: lc3_traits::peripherals::timers::TimerMode::Repeated
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
      [0x61] REPEATED,
      /// Puts a [Timer] in [Disabled] mode.
      ///
      /// ## Inputs
      ///  - [`R0`]: A [Timer] [ID] number.
      ///
      /// ## Outputs
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP sets the state of the [Timer] indicated by [`R0`] to
      /// [Disabled]. It does this by setting the period of the [Timer] to zero.
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid timer [ID] number, the `n` bit is
      /// set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T1`] to be a [Repeated] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. When [`T1`] fires an
      /// interrupt every second, `ISR` is called, which increments a counter.
      /// In the main program, the counter is checked repeatedly until the
      /// target of `10` is reached. After the target is reached, the timer is
      /// disabled and the program halts.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 1
      /// ADD R0, R0, #1
      /// LD R1, PERIOD       ; Sets R1 to 1000
      /// LEA R2, ISR         ; Sets R2 to the address of ISR
      /// TRAP 0x61           ; Sets T1 to Repeated w/ period of 1000 and ISR
      ///
      /// LD R1, TARGET       ; Sets R1 to -10
      /// LOOP
      /// LD R0, COUNTER      ; Loop until counter reaches 10
      /// ADD R0, R0, R1
      /// BRn LOOP
      ///
      /// AND R0, R0, #0      ; Sets R0 to 1
      /// ADD R0, R0, #1
      /// TRAP 0x62           ; Disable T1
      /// HALT
      ///
      /// PERIOD .FILL #1000
      /// COUNTER .FILL #0
      /// TARGET .FILL #-10
      ///
      /// ISR
      /// ADD R6, R6, #-1     ; Save R0 onto the supervisor stack
      /// STR R0, R6, #0
      ///
      /// LD R0, COUNTER
      /// ADD R0, R0, #1
      /// ST R0, COUNTER
      ///
      /// LDR R0, R6, #0      ; Pop R0 off of the supervisor stack
      /// ADD R6, R6, #1
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [Repeated]: lc3_traits::peripherals::timers::TimerMode::Repeated
      /// [Disabled]: lc3_traits::peripherals::timers::TimerState::Disabled
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T1`]: lc3_traits::peripherals::timers::TimerId::T1
      [0x62] DISABLE,
      /// Returns the mode of a [Timer].
      ///
      /// ## Inputs
      ///  - [`R0`]: A [Timer] [ID] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: A value corresponding to a [Timer] [mode].
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP returns the mode of the [Timer] indicated by [`R0`] by
      /// writing a value to [`R0`]. The values are as follows:
      ///
      /// | Mode           | Value |
      /// | -------------- | ----- |
      /// | [`SingleShot`] | 0     |
      /// | [`Repeated`]   | 1     |
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [SingleShot] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. It then gets the
      /// [mode] of [`T0`], which will write a value of 1 into [`R0`]. Then the
      /// program halts.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 1000
      /// LEA R2, ISR         ; Sets R2 to address of ISR
      /// TRAP 0x60           ; Sets T0 to SingleShot w/ period of 1000 and ISR
      /// TRAP 0x63           ; Reads T0's mode, sets R0 to 1
      /// HALT
      ///
      /// PERIOD .FILL #1000
      ///
      /// ISR                 ; Dummy ISR
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [`Repeated`]: lc3_traits::peripherals::timers::TimerMode::Repeated
      /// [`SingleShot`]: lc3_traits::peripherals::timers::TimerMode::SingleShot
      /// [SingleShot]: lc3_traits::peripherals::timers::TimerMode::SingleShot
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
      /// [mode]: lc3_traits::peripherals::timers::TimerMode
      [0x63] GET_MODE,
      /// Returns the period of a [Timer].
      ///
      /// ## Inputs
      ///  - [`R0`]: A [Timer] [ID] number.
      ///
      /// ## Outputs
      ///  - [`R0`]: A period ∈ \[0, 65535\].
      ///  - `n` bit: set on error, cleared on success.
      ///
      /// ## Usage
      ///
      /// This TRAP returns the period of the [Timer] indicated by [`R0`] by
      /// writing a value to [`R0`]. The period will be a value in the range
      /// \[0, 65535\] and is measured in milliseconds.
      ///
      /// Reading the period of a [Timer] that is [Disabled] will return a
      /// value of zero.
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [Repeated] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. It then gets the
      /// period of [`T0`], which will write a value of `1000` into [`R0`].
      /// Then the program halts.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// LD R1, PERIOD       ; Sets R1 to 1000
      /// LEA R2, ISR         ; Sets R2 to address of ISR
      /// TRAP 0x60           ; Sets T0 to SingleShot w/ period of 1000 and ISR
      /// TRAP 0x64           ; Reads T0's period, sets R0 to 1000
      /// HALT
      ///
      /// PERIOD .FILL #1000
      ///
      /// ISR                 ; Dummy ISR
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [SingleShot]: lc3_traits::peripherals::timers::TimerMode::SingleShot
      /// [Repeated]: lc3_traits::peripherals::timers::TimerMode::Repeated
      /// [Disabled]: lc3_traits::peripherals::timers::TimerState::Disabled
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
      /// [mode]: lc3_traits::peripherals::timers::TimerMode
      [0x64] GET_PERIOD,
  });

}

/// Trap vectors for the [`Clock`](lc3_traits::peripherals::Clock)
/// peripheral.
pub mod clock {
  define!([super::mm::MISC_OFFSET] <- {
      /// Sets the value of the [Clock].
      ///
      /// ## Inputs
      ///  - [`R0`]: Value to set the clock in milliseconds.
      ///
      /// ## Outputs
      ///  - None? TODO: check this
      ///
      /// ## Usage
      ///
      /// This TRAP sets the value of the [Clock] based on the value of [`R0`].
      /// The clock's value is measured in milliseconds and uses full 16-bit
      /// words (i.e. the value will be in the range \[0, 65535\]).
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below resets the [Clock]'s value:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0      ; Sets R0 to 0
      /// TRAP 0x70           ; Sets the Clock's value to 0
      /// ```
      ///
      /// [Clock]: lc3_traits::peripherals::clock
      /// [`R0`]: lc3_isa::Reg::R0
      [0x70] SET,
      /// Gets the value of the [Clock].
      ///
      /// ## Inputs
      ///  - None
      ///
      /// ## Outputs
      ///  - [`R0`]: Value of the clock.
      ///
      /// ## Usage
      ///
      /// This TRAP gets the value of the [Clock] and stores it in [`R0`].
      /// The clock's value is measured in milliseconds and uses full 16-bit
      /// words (i.e. the value will be in the range \[0, 65535\]).
      ///
      /// All registers (**excluding** [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below gets the [Clock]'s value and stores it in [`R0`]:
      /// ```{ARM Assembly}
      /// TRAP 0x71       ; Gets the Clock's value, sets R0
      /// ```
      ///
      /// [Clock]: lc3_traits::peripherals::clock
      /// [`R0`]: lc3_isa::Reg::R0
      [0x71] GET,
  });
}

/// Trap vectors for the [`Input`](lc3_traits::peripherals::Input)
/// peripheral.
pub mod input {
  pub use super::builtin::GETC as READ;
}

/// Trap vectors for the [`Output`](lc3_traits::peripherals::Output)
/// peripheral.
pub mod output {
  pub use super::builtin::OUT as WRITE;
}

/// Trap vectors for the Traps officially part of the LC-3 ISA (i.e. [GETC],
/// [OUT], [PUTS], [IN], [HALT], etc.).
///
/// [GETC]: builtin::GETC
/// [OUT]: builtin::OUT
/// [PUTS]: builtin::PUTS
/// [IN]: builtin::IN
/// [HALT]: builtin::HALT
pub mod builtin {
  define!([0x20] <- {
      /// Reads a character from the keyboard.
      ///
      /// This TRAP will block until a character is available from the [`Input`]
      /// peripheral (a.k.a. the keyboard).
      ///
      /// ## Inputs
      ///  - None
      ///
      /// ## Outputs
      ///  - [`R0`]: Character read from keyboard.
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing Systems:
      /// From Bits and Gates to C and Beyond (Patt and Patel)*,
      /// [Appendix A](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf)
      /// [Page 23](http://www.cs.unca.edu/~bruce/Spring14/109/Resources/lc3-isa.pdf#page=23).
      ///
      /// Read a single character from the keyboard. The character is not
      /// echoed onto the console. Its ASCII code is copied into R0. The high
      /// eight bits of R0 are cleared.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`Input`]: lc3_traits::peripherals::input::Input
      [0x20] GETC,   // 0x20
      /// Writes a character to the console display.
      ///
      /// This TRAP will block until the [`Output`] peripheral (a.k.a. the
      /// display) has accepted the character.
      ///
      /// ## Inputs
      ///  - [`R0`]: Character to write.
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing Systems:
      /// From Bits and Gates to C and Beyond (Patt and Patel)*,
      /// [Appendix A](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf)
      /// [Page 23](http://www.cs.unca.edu/~bruce/Spring14/109/Resources/lc3-isa.pdf#page=23).
      ///
      ///  Write a character in R0\[7:0\] to the console display.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`Output`]: lc3_traits::peripherals::output::Output
      [0x21] OUT,    // 0x21
      /// Writes a string of ASCII characters to the console display.
      ///
      /// This TRAP will block until the [`Output`] peripheral (a.k.a. the
      /// display) has accepted the all the characters in the string.
      ///
      /// ## Inputs
      ///  - [`R0`]: Address of first character.
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing Systems:
      /// From Bits and Gates to C and Beyond (Patt and Patel)*,
      /// [Appendix A](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf)
      /// [Page 23](http://www.cs.unca.edu/~bruce/Spring14/109/Resources/lc3-isa.pdf#page=23).
      ///
      /// Write a string of ASCII characters to the console display. The
      /// characters are contained in consecutive memory locations, one
      /// character per memory location, starting with the address specified in
      /// [`R0`]. Writing terminates with the occurrence of `0x0000` in a memory
      /// location.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`Output`]: lc3_traits::peripherals::output::Output
      [0x22] PUTS,   // 0x22
      /// Prints a prompt and reads a character from the keyboard.
      ///
      /// This TRAP will block until the [`Output`] peripheral has accepted
      /// all the characters in the prompt **and** then until the [`Input`]
      /// peripheral provides a character.
      ///
      /// ## Inputs
      ///  - None
      ///
      /// ## Outputs
      ///  - [`R0`]: Character read from keyboard.
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing Systems:
      /// From Bits and Gates to C and Beyond (Patt and Patel)*,
      /// [Appendix A](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf)
      /// [Page 23](http://www.cs.unca.edu/~bruce/Spring14/109/Resources/lc3-isa.pdf#page=23).
      ///
      /// Print a prompt on the screen and read a single character from the
      /// keyboard. The character is echoed onto the console monitor, and its
      /// ASCII code is copied into [`R0`]. The high eight bits of [`R0`] are
      /// cleared.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`Output`]: lc3_traits::peripherals::output::Output
      /// [`Input`]: lc3_traits::peripherals::input::Input
      [0x23] IN,     // 0x23
      /// Writes a string of ASCII characters, stored compactly, to the
      /// console display.
      ///
      /// This TRAP will block until the [`Output`] peripheral (a.k.a. the
      /// display) has accepted the all the characters in the string.
      ///
      /// ## Inputs
      ///  - [`R0`]: Address of first characters.
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing Systems:
      /// From Bits and Gates to C and Beyond (Patt and Patel)*,
      /// [Appendix A](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf)
      /// [Page 23](http://www.cs.unca.edu/~bruce/Spring14/109/Resources/lc3-isa.pdf#page=23).
      ///
      /// Write a string of ASCII characters to the console. The characters are
      /// contained in consecutive memory locations, two characters per memory
      /// location, starting with the address specified in [`R0`]. The ASCII
      /// code contained in bits \[7:0\] of a memory location is written to the
      /// console first. Then the ASCII code contained in bits \[15:8\] of that
      /// memory location is written to the console. (A character string
      /// consisting of an odd number of characters to be written will have
      /// `0x00` in bits \[15:8\] of the memory location containing the last
      /// character to be written.) Writing terminates with the occurrence of
      /// `0x0000` in a memory location.
      ///
      /// Note that our (and [lc3tools]') PUTSP implementation is more lenient
      /// than this; if you have an odd number of characters, the `0x00` in
      /// bits \[15:8\] of the last word is sufficient to terminate the string
      /// and the last `0x0000` word can be omitted (we just look for the first
      /// 0x00 character to terminate).
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`Output`]: lc3_traits::peripherals::output::Output
      /// [lc3tools]: https://github.com/chiragsakhuja/lc3tools
      [0x24] PUTSP,  // 0x24
      /// Halt execution and print a message on the console.
      ///
      /// ## Inputs
      ///  - None
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing Systems:
      /// From Bits and Gates to C and Beyond (Patt and Patel)*,
      /// [Appendix A](http://highered.mheducation.com/sites/dl/free/0072467509/104691/pat67509_appa.pdf)
      /// [Page 23](http://www.cs.unca.edu/~bruce/Spring14/109/Resources/lc3-isa.pdf#page=23).
      ///
      /// Halt execution and print a message on the console.
      [0x25] HALT,   // 0x25
  });
}
