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

        //Consider case where the time taken for an iteration in this loop to match and put an element in Fifo exceeds the limit
        //to collect bytes from the UART hardware buffer. This could happen when the ratio of UART baud rate to clock frequency
        //the device operates at exceeds a certain threshold. To an extent this can be alleviated by having hardware buffers like the
        //TM4C has (16 elements), but that would eventually overflow for large messages, and some devices such as MSP430s have 
        //just 1 element hardware buffers. This problem could potentially be solved by DMA. Seems like one use case of dma to 
        //support high baud rates relativistic to processor speed.

        // Apart from that, this looks like a good approach. Not blocking so not dependent on having the full message available in
        // 1 invocation of get. Can keep updating the buffer among multipe invocations of this loop till the sentinel is obtained.,
        // However, that risks dropping bytes. it might be possible to miss data if that's the case. Consider this -> get is called
        // and it puts a byte into the Fifo. But suppose that was the only byte available (or if it read all available ytes in the device
        // hardware buffer), then returns a wouldBlock error on the next iteration of loop, it would exit this function and suppose,
        // the main event loop goes to do another task like updating peripheral data/reading sensors... before it invokes get again,
        // then once it calls get again, it could be the case that there are more bytes "within" the hardware buffer limit of device to read
        // and no overflow occured which is good, or it could be that the event loop was too slow to call get again and bytes are lost
        // This is very likely in devices that just have a single element buffer. DMA and interrupts could solve this problem by esentially having
        // an extended hardware buffer of whatever size we prefer by directly writing to memory and this process is not dependent on/done by the foreground thread

        // So while DMA/interrupts don't necessarily increase the "speed" of data collection and processing, the main adavtage
        // seems to be in freeing up processor from this load and having processor so more useful things instead and not worry about
        // precise timing of collection of UART data before they are lost thus increasing baud rate to bus clock frequency ratio. Especially useful if running a multithreaded RTOS where the processor
        // might have to do other stuff like updated peripheral sensors data...DMA however also has its own issues, This UART polling is a good implementation for a lot of use cases so it depends on weighing
        // the tradeoffs on specific device being used, the baud rate, clock frequency, hardware Fifo length...
        // 
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
