//! TODO!

use crate::io_peripherals::{InputSink, OutputSource};

use lc3_traits::peripherals::PeripheralSet;
use lc3_shims::peripherals::{Source, Sink};
use lc3_shims::peripherals::{GpioShim, AdcShim, PwmShim, TimersShim, ClockShim, InputShim, OutputShim};

use std::sync::{Arc, Mutex, RwLock};

pub struct Shims<'int> {
    pub gpio: Arc<RwLock<GpioShim<'int>>>,
    pub adc: Arc<RwLock<AdcShim>>,
    pub pwm: Arc<RwLock<PwmShim>>,
    pub timers: Arc<RwLock<TimersShim<'int>>>,
    pub clock: Arc<RwLock<ClockShim>>,
}

pub type ShimPeripheralSet<'int: 'io, 'io> = PeripheralSet<
    'int,
    Arc<RwLock<GpioShim<'int>>>,
    Arc<RwLock<AdcShim>>,
    Arc<RwLock<PwmShim>>,
    Arc<RwLock<TimersShim<'int>>>,
    Arc<RwLock<ClockShim>>,
    Arc<Mutex<InputShim<'io, 'int>>>,
    Arc<Mutex<OutputShim<'io, 'int>>>,
>;

pub fn new_shim_peripherals_set<'int: 'io, 'io, I, O>(input: &'io I, output: &'io O)
        -> (ShimPeripheralSet<'int, 'io>, &'io impl InputSink, &'io impl OutputSource)
where
    I: InputSink + Source + Send + Sync + 'io,
    O: OutputSource + Sink + Send + Sync + 'io,
{
    let gpio_shim = Arc::new(RwLock::new(GpioShim::default()));
    let adc_shim = Arc::new(RwLock::new(AdcShim::default()));
    let pwm_shim = Arc::new(RwLock::new(PwmShim::default()));
    let timer_shim = Arc::new(RwLock::new(TimersShim::default()));
    let clock_shim = Arc::new(RwLock::new(ClockShim::default()));

    let input_shim = Arc::new(Mutex::new(InputShim::with_ref(input)));
    let output_shim = Arc::new(Mutex::new(OutputShim::with_ref(output)));

    (PeripheralSet::new(gpio_shim, adc_shim, pwm_shim, timer_shim, clock_shim, input_shim, output_shim),
        input,
        output,
    )
}

impl<'int> Shims<'int> {
    pub fn from_peripheral_set<'io>(p: &ShimPeripheralSet<'int, 'io>) -> Self
    where
        'int: 'io
    {
        Self {
            gpio: p.get_gpio().clone(),
            adc: p.get_adc().clone(),
            pwm: p.get_pwm().clone(),
            timers: p.get_timers().clone(),
            clock: p.get_clock().clone(),
        }
    }
}
