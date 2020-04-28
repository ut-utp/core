//! UART transport for computers.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};

use mio_serial::{
    Serial, DataBits, FlowControl, Parity, StopBits, SerialPort
};
pub use mio_serial::SerialPortSettings;

use std::path::Path;
use std::io::Result as IoResult;
use std::io::{Read, Write, Error, ErrorKind};
use std::convert::AsRef;
use std::cell::RefCell;
use std::time::Duration;

#[derive(Debug)]
pub struct HostUartTransport {
    serial: RefCell<Serial>,
    internal_buffer: RefCell<Fifo<u8>>,
}

impl HostUartTransport {
    pub fn new<P: AsRef<Path>>(path: P, baud_rate: u32) -> IoResult<Self> {
        let settings = SerialPortSettings {
            baud_rate: baud_rate,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_secs(100),
        };

        Self::new_with_config(path, settings)
    }

    pub fn new_with_config<P: AsRef<Path>>(path: P, config: SerialPortSettings) -> IoResult<Self> {
        let serial = Serial::from_path(path, &config)?;

        Ok(Self {
            serial: RefCell::new(serial),
            internal_buffer: RefCell::new(Fifo::new_const()),
        })
    }
}

// TODO: on std especially we don't need to pass around buffers; we can be
// zero-copy...
impl Transport<Fifo<u8>, Fifo<u8>> for HostUartTransport {
    type RecvErr = Error;
    type SendErr = Error;

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("host");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, message: Fifo<u8>) -> IoResult<()> {
        let mut serial = self.serial.borrow_mut();

        use std::io::ErrorKind;

        macro_rules! block {
            ($e:expr) => {
                loop {
                    match $e {
                        Ok(()) => break IoResult::Ok(()),
                        Err(e) => match e.kind() {
                            ErrorKind::WouldBlock => continue,
                            _ => return Err(e),
                        }
                    }
                }
            };
        }

        // serial.write(message.as_slice()).map(|_| ())?;
        // serial.flush()

        block!(serial.write(message.as_slice()).map(|_| ()));
        block!(serial.flush())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<Error>> {
        let mut serial = self.serial.borrow_mut();
        let mut buf = self.internal_buffer.borrow_mut();

        // Note: this is bad!

        let mut temp_buf = [0; 1];

        loop {
            match serial.read(&mut temp_buf) {
                Ok(1) => {
                    if temp_buf[0] == 0 {
                        break Ok(core::mem::replace(&mut buf, Fifo::new()))
                    } else {
                        // TODO: don't panic here; see the note in uart_simple
                        buf.push(temp_buf[0]).unwrap()
                    }
                },
                Ok(0) => {},
                Ok(_) => unreachable!(),
                Err(err) => {
                    if let ErrorKind::WouldBlock = err.kind() {
                        break Err(None)
                    } else {
                        break Err(Some(err))
                    }
                }
            }
        }
    }
}


