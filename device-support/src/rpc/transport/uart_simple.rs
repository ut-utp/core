//! A primitive, simplistic, [`Transport`] impl that uses UART.
//!
//! This does not use interrupts or DMA and does not attempt to be zero-copy.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};

use embedded_hal::serial::{Read, Write};
use nb::block;

use core::cell::RefCell;
use core::fmt::Debug;

#[derive(Debug)]
pub struct UartTransport<R: Read<u8>, W: Write<u8>>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    read: RefCell<R>,
    write: RefCell<W>,
    internal_buffer: RefCell<Fifo<u8>>,
}

impl<R: Read<u8>, W: Write<u8>> UartTransport<R, W>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    // Can't be const until bounds are allowed.
    pub /*const*/ fn new(read: R, write: W) -> Self {
        Self {
            read: RefCell::new(read),
            write: RefCell::new(write),
            internal_buffer: RefCell::new(Fifo::new_const()),
        }
    }
}

impl<R: Read<u8>, W: Write<u8>> Transport<Fifo<u8>, Fifo<u8>> for UartTransport<R, W>
where
    <R as Read<u8>>::Error: Debug,
    <W as Write<u8>>::Error: Debug,
{
    type RecvErr = R::Error;
    type SendErr = W::Error;

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("simp");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, message: Fifo<u8>) -> Result<(), W::Error> {
        let mut write = self.write.borrow_mut();

        for byte in message {
            block!(write.write(byte))?
        }

        block!(write.flush())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<R::Error>> {
        let mut read = self.read.borrow_mut();
        let mut buf = self.internal_buffer.borrow_mut();

        use nb::Error::*;

        loop {
            match read.read() {
                Ok(word) => {
                    if word == 0 {
                        // 0 is the sentinel!
                        break Ok(core::mem::replace(&mut buf, Fifo::new()))
                    } else {
                        // TODO: don't panic here, just dump the buffer or
                        // something.
                        buf.push(word).unwrap()
                    }
                },
                Err(WouldBlock) => {
                    break Err(None)
                },
                Err(Other(err)) => {
                    break Err(Some(err))
                }
            }
        }
    }
}
