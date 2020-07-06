//! TODO!

use super::{BlackBox, Init};
use crate::shim_support::Shims;

use lc3_shims::peripherals::SourceShim;
use lc3_traits::control::rpc::{
    futures::SyncEventFutureSharedState,
    Controller, RequestMessage, ResponseMessage,
};
use lc3_device_support::{
    rpc::{
        transport::uart_host::{HostUartTransport, SerialPortSettings},
        encoding::{PostcardEncode, PostcardDecode, Cobs},
    },
    util::Fifo,
};

use std::{
    sync::Mutex,
    thread::Builder as ThreadBuilder,
    path::{Path, PathBuf},
    default::Default,
    marker::PhantomData,
};

// Static data that we need:
// TODO: note that, like sim and sim_rpc, this will cause problems if more than
// 1 instance of this simulator is instantiated.
lazy_static::lazy_static! {
    pub static ref EVENT_FUTURE_SHARED_STATE_CONT: SyncEventFutureSharedState =
        SyncEventFutureSharedState::new();
}

type Cont<'ss, EncFunc: FnMut() -> Cobs<Fifo<u8>>> = Controller<
    'ss,
    HostUartTransport,
    SyncEventFutureSharedState,
    RequestMessage,
    ResponseMessage,
    PostcardEncode<RequestMessage, Cobs<Fifo<u8>>, EncFunc>,
    PostcardDecode<ResponseMessage, Cobs<Fifo<u8>>>,
>;

// #[derive(Debug)]
pub struct BoardDevice<'ss, EncFunc = Box<dyn FnMut() -> Cobs<Fifo<u8>>>, P = &'static Path>
where
    EncFunc: FnMut() -> Cobs<Fifo<u8>>,
    P: AsRef<Path>,
{
    controller: Cont<'ss, EncFunc>,
    _p: PhantomData<P>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerialSettings {
    DefaultsWithBaudRate { baud_rate: u32 },
    Custom(SerialPortSettings),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardConfig<P: AsRef<Path>> {
    pub path: P,
    pub serial_settings: SerialSettings,
}

// TODO: use Strings instead?
impl<P: AsRef<Path>> Default for BoardConfig<&'static P>
where
    &'static P: AsRef<Path>,
    P: 'static,
    str: AsRef<P>
{
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        { Self::detect_windows() }

        #[cfg(target_os = "macos")]
        { Self::detect_macos() }

        #[cfg(target_os = "linux")]
        { Self::detect_linux() }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { Self::new(AsRef::<P>::as_ref("/dev/tm4c"), 1_500_000) }
    }
}

// TODO: actually have this use the platform stuff (spin off the functions below into
// things that just return a PathBuf, I think).
impl Default for BoardConfig<PathBuf> {
    fn default() -> Self {
        Self::new(PathBuf::from("/dev/lm4f"), 1_500_000)
    }
}

// TODO: port over [this script]
//
// [this script](https://github.com/ut-ras/Rasware/blob/f3750ff0b8f7f7791da2fee365462d4c78f62a49/RASLib/detect-board)

impl<P: AsRef<Path>> BoardConfig<&'static P>
where
    &'static P: AsRef<Path>,
    P: 'static,
    str: AsRef<P>
{
    #[cfg(target_os = "windows")]
    pub fn detect_windows() -> Self {
        unimplemented!()
    }

    #[cfg(target_family = "unix")]
    pub fn detect_linux() -> Self {
        Self::new(AsRef::<P>::as_ref("/dev/lm4f"), 1_500_000)
    }

    #[cfg(target_os = "macos")]
    pub fn detect_macos() -> Self {
        unimplemented!()
    }
}

impl<P: AsRef<Path>> BoardConfig<P> {
    pub /*const*/ fn new(path: P, baud_rate: u32) -> Self {
        Self {
            path,
            serial_settings: SerialSettings::DefaultsWithBaudRate { baud_rate },
        }
    }

    pub /*const*/ fn new_with_config(path: P, config: SerialPortSettings) -> Self {
        Self {
            path,
            serial_settings: SerialSettings::Custom(config),
        }
    }
}

impl<P: AsRef<Path>> BoardConfig<P> {
    fn new_transport(self) -> HostUartTransport {
        // Note: we unwrap here! This is probably not great!
        // (TODO)
        match self.serial_settings {
            SerialSettings::DefaultsWithBaudRate { baud_rate } => {
                HostUartTransport::new(self.path, baud_rate).unwrap()
            },

            SerialSettings::Custom(config) => {
                HostUartTransport::new_with_config(self.path, config).unwrap()
            },
        }
    }
}

impl<'s, P> Init<'s> for BoardDevice<'static, Box<dyn FnMut() -> Cobs<Fifo<u8>>>, P>
where
    P: AsRef<Path>,
    P: 'static,
    BoardConfig<P>: Default,
{
    type Config = BoardConfig<P>;

    // Until we get existential types (or `impl Trait` in type aliases) we can't
    // just say a type parameter is some specific type that we can't name (i.e.
    // a particular function).
    //
    // Instead we have to name it explicitly, so we do this unfortunate hack:
    type ControlImpl = Cont<'static, Box<dyn FnMut() -> Cobs<Fifo<u8>>>>;

    type Input = SourceShim; // TODO
    type Output = Mutex<Vec<u8>>; // TODO

    fn init_with_config(
        b: &'s mut BlackBox,
        config: BoardConfig<P>,
    ) -> (
        &'s mut Self::ControlImpl,
        Option<Shims<'static>>,
        Option<&'s Self::Input>,
        Option<&'s Self::Output>,
    ) {
        let func: Box<dyn FnMut() -> Cobs<Fifo<u8>>> = Box::new(|| Cobs::try_new(Fifo::new()).unwrap());

        let controller = Controller::new(
            PostcardEncode::new(func),
            PostcardDecode::new(),
            config.new_transport(),
            &*EVENT_FUTURE_SHARED_STATE_CONT
        );

        let storage: &'s mut _ = b.put(BoardDevice::<_, P> { controller, _p: PhantomData });

        (
            &mut storage.controller,
            None,
            None, // TODO
            None, // TODO
        )
    }
}
