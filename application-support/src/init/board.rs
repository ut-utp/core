//! TODO!

use super::{BlackBox, Init};
use crate::{
    shim_support::Shims,
};

use lc3_traits::control::rpc::{
    futures::SyncEventFutureSharedState,
    Controller, RequestMessage, ResponseMessage,
};
use lc3_device_support::{
    transport::uart_host::{HostUartTransport, SerialPortSettings},
    encoding::{PostcardEncode, PostcardDecode},
    util::Fifo,
};

use std::{
    sync::Mutex,
    thread::Builder as ThreadBuilder,
    path::Path,
    default::Default,
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

#[derive(Debug)]
pub struct BoardDevice<'ss, EncFunc: FnMut() -> Cobs<Fifo<u8>>> {
    controller: Cont<'ss, EncFunc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SerialSettings {
    DefaultsWithBaudRate { baud_rate: u32 },
    Custom(SerialPortSettings),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoardConfig<P: AsRef<Path>> {
    path: P,
    serial_settings: SerialSettings,
}

impl<P: AsRef<Path>> Default for BoardConfig<P>
where
    str: AsRef<P>
{
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        Self::detect_windows()

        #[cfg(target_os = "macos")]
        Self::detect_macos()

        #[cfg(target_os = "linux")]
        Self::detect_linux()

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Self::new(AsRef::<P>::as_ref("/dev/tm4c"), 1_500_000)
    }
}

// TODO: port over [this script]
//
// [this script](https://github.com/ut-ras/Rasware/blob/f3750ff0b8f7f7791da2fee365462d4c78f62a49/RASLib/detect-board)

impl<P: AsRef<Path>> BoardConfig<P>
where
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
    pub const fn new<P>(path: P, baud_rate: u32) -> Self {
        Self {
            path,
            serial_settings: SerialSettings::DefaultsWithBaudRate(baud_rate),
        }
    }

    pub const fn new_with_config<P: AsRef<Path>>(path: P, config: SerialPortSettings) -> Self {
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
                HostUartTransport::new(self.path, baud_rate)
            },

            SerialSettings::Custom(config) => {
                HostUartTransport::new_with_config(self.path, config)
            },
        }
    }
}

impl<'s, P, EncFunc> Init<'s> for BoardDevice<'static, EncFunc>
where
    P: AsRef<Path>,
    BoardConfig<P>: Default,
    EncFunc: FnMut() -> Cobs<Fifo<u8>>,
{
    type Config = BoardConfig<P>;

    type ControlImpl = Cont<'static, EncFunc>;
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
        let controller = Controller::new(
            PostcardEncode::with_fifo(),
            PostcardDecode::new(),
            config.new_transport(),
            &EVENT_FUTURE_SHARED_STATE_CONT
        );

        let storage: &'s mut _ = b.put(BoardDevice { controller });

        (
            &mut storage.controller,
            None,
            None, // TODO
            None, // TODO
        )
    }
}
