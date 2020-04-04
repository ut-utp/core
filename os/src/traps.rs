//! Trap vector numbers and documentation.
//!
//! ### Guidelines on Writing ISRs
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
      /// This TRAP puts the [GPIO] [Pin] indicated by [`R0`] into [Input] mode.
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is ∈ \[0,
      /// [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Input]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x30
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
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is ∈
      /// \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x31
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
      /// All registers (including [`R0`]) are preserved.
      ///
      /// Be sure to follow the
      /// [guidelines for writing ISRs](../index.html#guidelines-on-writing-isrs).
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Interrupt] and sets the interrupt
      /// service routine to `ISR`. It will then spin until [`G0`] fires an
      /// interrupt, which will call `ISR`, set the `ISR_FLAG` to 1, and allow
      /// the main program to halt.
      ///
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LEA R1, ISR
      /// TRAP 0x32
      ///
      /// LOOP
      /// LD R1, ISR_FLAG
      /// BRz LOOP
      /// HALT
      ///
      /// ISR_FLAG .FILL #0
      ///
      /// ISR
      /// AND R0, R0, #0
      /// ADD R0, R0, #1
      /// ST R0, ISR_FLAG
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
      /// All registers (including [`R0`] and [`R1`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output], then immediately sets it
      /// to [Disabled]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x31
      /// TRAP 0x33
      ///
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
      /// Returns the [mode] of a [GPIO] [Pin].
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
      /// This TRAP returns the [mode] of the [GPIO] [Pin] indicated by [`R0`] by
      /// writing a value to [`R0`]. The values are as follows:
      ///
      /// | Mode          | Value |
      /// | ------------- | ----- |
      /// | [`Input`]     | 0     |
      /// | [`Output`]    | 1     |
      /// | [`Interrupt`] | 2     |
      /// | [`Disabled`]  | 3     |
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_GPIO_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output], then reads [`G0`]'s mode
      /// into [`R0`]. [`R0`] will then contain the value 2.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x31
      /// TRAP 0x34
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
      /// Attempting to write to a [GPIO] [Pin] that is not in [Output] mode does
      /// nothing.
      ///
      /// All registers (including [`R0`] and [`R1`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Output], then writes the value 1 to [`G0`]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x31
      /// AND R1, R1, #0
      /// ADD R1, R1, #1
      /// TRAP 0x35
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
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`G0`] to be an [Input], then reads from [`G0`] into [`R0`]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x30
      /// TRAP 0x36
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
      /// AND R0, R0, #0
      /// TRAP 0x40
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
      /// AND R0, R0, #0
      /// TRAP 0x40
      /// TRAP 0x41
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
      /// | [`Enabled`]   | 0     |
      /// | [`Disabled`]  | 1     |
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_ADC_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`A0`] to be an [Disabled], then reads [`A0`]'s mode
      /// into [`R0`]. [`R0`] will then contain the value 1.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x41
      /// TRAP 0x42
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
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`A0`] to be an [Enabled], then reads from [`A0`] into [`R0`]:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// TRAP 0x40
      /// TRAP 0x43
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
      /// the fractional value (e.g. a value of 64 corresponds to a 25% duty cycle).
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_PWM_PINS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (including [`R0`], [`R1`], and [`R2`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and a
      /// *50%* duty cycle then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LD R2, DUTY
      /// TRAP 0x50
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
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and a
      /// *50%* duty cycle, immediately sets it to [Disabled], then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LD R2, DUTY
      /// TRAP 0x50
      /// TRAP 0x51
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
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and a
      /// *50%* duty cycle. It then reads the period of [`P0`] and results in the
      /// value 20 in [`R0`] then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LD R2, DUTY
      /// TRAP 0x50
      /// TRAP 0x52
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
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`P0`] to be an [Enabled] with a period of *20 ms* and a
      /// *50%* duty cycle. It then reads the duty cycle of [`P0`] and results in
      /// the value 128 in [`R0`] then halts:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LD R2, DUTY
      /// TRAP 0x50
      /// TRAP 0x53
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
      /// triggered one time. The [Timer] will then set itself to [Disabled].
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid timer [ID] number, the `n` bit is set.
      ///
      /// All registers (including [`R0`], [`R1`], and [`R2`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [SingleShot] with a period of `3 seconds`
      /// and sets the interrupt service routine to `ISR`. It will then spin
      /// until [`T0`] fires an interrupt, after three seconds have passed,
      /// which will call `ISR`, set the `ISR_FLAG` to 1, and allow the main
      /// program to halt.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LEA R2, ISR
      /// TRAP 0x60
      ///
      /// LOOP
      /// LD R1, ISR_FLAG
      /// BRz LOOP
      /// HALT
      ///
      /// PERIOD .FILL #3000
      /// ISR_FLAG .FILL #0
      ///
      /// ISR
      /// AND R0, R0, #0
      /// ADD R0, R0, #1
      /// ST R0, ISR_FLAG
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [Disabled]: lc3_traits::peripherals::timers::TimerState::Disabled
      /// [SingleShot]: lc3_traits::peripherals::timers::TimerState::SingleShot
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
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid timer [ID] number, the `n` bit is set.
      ///
      /// All registers (including [`R0`], [`R1`], and [`R2`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [Repeated] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. It will then spin
      /// endlessly. When [`T0`] fires an interrupt every second, `ISR` is called,
      /// which increments a counter:
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LEA R2, ISR
      /// TRAP 0x61
      ///
      /// LOOP
      /// BRz LOOP
      ///
      /// PERIOD .FILL #1000
      /// COUNTER .FILL #0
      ///
      /// ISR
      /// LD R0, COUNTER
      /// ADD R0, R0, #1
      /// ST R0, COUNTER
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [Repeated]: lc3_traits::peripherals::timers::TimerState::Repeated
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
      /// This TRAP puts the [Timer] indicated by [`R0`] into [Disabled]
      /// mode. When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid timer [ID] number, the `n` bit is set.
      ///
      /// All registers (including [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [Repeated] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`, then immediately
      /// disables [`T0`] It will then spin endlessly. Since [`T0`] is disabled,
      /// `ISR` is never called, and the counter never increments.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LEA R2, ISR
      /// TRAP 0x61
      ///
      /// TRAP 0x62
      ///
      /// LOOP
      /// BRz LOOP
      ///
      /// PERIOD .FILL #1000
      /// COUNTER .FILL #0
      ///
      /// ISR
      /// LD R0, COUNTER
      /// ADD R0, R0, #1
      /// ST R0, COUNTER
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [Repeated]: lc3_traits::peripherals::timers::TimerState::Repeated
      /// [Disabled]: lc3_traits::peripherals::timers::TimerState::Disabled
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
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
      /// | [`Repeated`]   | 0     |
      /// | [`SingleShot`] | 1     |
      /// | [`Disabled`]   | 2     |
      ///
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [SingleShot] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. It then gets the
      /// [mode] of [`T0`], which will write a value of 1 into [`R0`]. Then the
      /// program halts.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LEA R2, ISR
      /// TRAP 0x60
      /// TRAP 0x63
      /// HALT
      ///
      /// PERIOD .FILL #1000
      ///
      /// ISR
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [`Repeated`]: lc3_traits::peripherals::timers::TimerState::Repeated
      /// [`SingleShot`]: lc3_traits::peripherals::timers::TimerState::SingleShot
      /// [`Disabled`]: lc3_traits::peripherals::timers::TimerState::Disabled
      /// [SingleShot]: lc3_traits::peripherals::timers::TimerState::SingleShot
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
      /// [mode]: lc3_traits::peripherals::timers::TimerState
      [0x63] GET_MODE,
      /// Returns the period of a [Timer] in [SingleShot] or [Repeated] mode.
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
      /// When [`R0`] contains a valid pin number (i.e. when [`R0`] is
      /// ∈ \[0, [`NUM_TIMERS`])), this TRAP is _infallible_.
      ///
      /// When [`R0`] does not hold a valid pin number, the `n` bit is set.
      ///
      /// Attempting to read the period of a [Timer] not in [SingleShot] or
      /// [Repeated] modes will return -1.
      ///
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below sets [`T0`] to be a [Repeated] with a period of `1 second`
      /// and sets the interrupt service routine to `ISR`. It then gets the
      /// period of [`T0`], which will write a value of `1000` into [`R0`].
      /// Then the program halts.
      /// ```{ARM Assembly}
      /// AND R0, R0, #0
      /// LD R1, PERIOD
      /// LEA R2, ISR
      /// TRAP 0x60
      /// TRAP 0x64
      /// HALT
      ///
      /// PERIOD .FILL #1000
      ///
      /// ISR
      /// RTI
      /// ```
      ///
      /// [Timer]: lc3_traits::peripherals::timers
      /// [SingleShot]: lc3_traits::peripherals::timers::TimerState::SingleShot
      /// [Repeated]: lc3_traits::peripherals::timers::TimerState::Repeated
      /// [ID]: lc3_traits::peripherals::timers::TimerId
      /// [`R0`]: lc3_isa::Reg::R0
      /// [`R1`]: lc3_isa::Reg::R1
      /// [`R2`]: lc3_isa::Reg::R2
      /// [`NUM_TIMERS`]: lc3_traits::peripherals::timers::TimerId::NUM_TIMERS
      /// [`T0`]: lc3_traits::peripherals::timers::TimerId::T0
      /// [mode]: lc3_traits::peripherals::timers::TimerState
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
      /// AND R0, R0, #0
      /// TRAP 0x70
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
      /// All registers (excluding [`R0`]) are preserved.
      ///
      /// ## Example
      /// The below gets the [Clock]'s value and stores it in [`R0`]:
      /// ```{ARM Assembly}
      /// TRAP 0x71
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
      /// ## Inputs
      ///  - None
      ///
      /// ## Outputs
      ///  - [`R0`]: Character read from keyboard.
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing
      /// Systems: From Bits and Gates to C and Beyond (Patt and Patel)*.
      ///
      /// Read a single character from the keyboard. The character is not
      /// echoed onto the console. Its ASCII code is copied into R0. The high
      /// eight bits of R0 are cleared.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      [0x20] GETC,   // 0x20
      /// Writes a character to the console display.
      ///
      /// ## Inputs
      ///  - [`R0`]: Character to write.
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing
      /// Systems: From Bits and Gates to C and Beyond (Patt and Patel)*.
      ///
      ///  Write a character in R0\[7:0\] to the console display.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      [0x21] OUT,    // 0x21
      /// Writes a string of ASCII characters to the console display.
      ///
      /// ## Inputs
      ///  - [`R0`]: Address of first character.
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing
      /// Systems: From Bits and Gates to C and Beyond (Patt and Patel)*.
      ///
      /// Write a string of ASCII characters to the console display.
      /// The characters are contained in consecutive memory locations,
      /// one character per memory location, starting with the address
      /// specified in R0. Writing terminates with the occurrence of
      /// x0000 in a memory location.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      [0x22] PUTS,   // 0x22
      /// Prints a prompt and reads a character from the keyboard.
      ///
      /// ## Inputs
      ///  - None
      ///
      /// ## Outputs
      ///  - [`R0`]: Character read from keyboard.
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing
      /// Systems: From Bits and Gates to C and Beyond (Patt and Patel)*.
      ///
      /// Print a prompt on the screen and read a single character from
      /// the keyboard. The character is echoed onto the console monitor,
      /// and its ASCII code is copied into R0. The high eight bits of R0
      /// are cleared.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
      [0x23] IN,     // 0x23
      /// Writes a string of ASCII characters stored compactly to the
      /// console display.
      ///
      /// ## Inputs
      ///  - [`R0`]: Address of first characters.
      ///
      /// ## Outputs
      ///  - None
      ///
      /// ## Usage
      ///
      /// The following description is from *Introduction to Computing
      /// Systems: From Bits and Gates to C and Beyond (Patt and Patel)*.
      ///
      /// Write a string of ASCII characters to the console. The
      /// characters are contained in consecutive memory locations, two
      /// characters per memory location, starting with the address
      /// specified in R0. The ASCII code contained in bits \[7:0\] of a
      /// memory location is written to the console first. Then the ASCII
      /// code contained in bits \[15:8\] of that memory location is
      /// written to the console. (A character string consisting of an
      /// odd number of characters to be written will have x00 in bits
      /// \[15:8\] of the memory location containing the last character to
      /// be written.) Writing terminates with the occurrence of x0000
      /// in a memory location.
      ///
      /// [`R0`]: lc3_isa::Reg::R0
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
      /// The following description is from *Introduction to Computing
      /// Systems: From Bits and Gates to C and Beyond (Patt and Patel)*.
      ///
      /// Halt execution and print a message on the console.
      [0x25] HALT,   // 0x25
  });
}
