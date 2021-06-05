//! This "attempts" to use DMA (device impl can)and does not attempt to be zero-copy.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};

use embedded_dma::{StaticReadBuffer, StaticWriteBuffer};
use bbqueue::{BBBuffer, GrantR, GrantW, ConstBBBuffer, Consumer, Producer, consts::*};
use nb::block;

use core::cell::RefCell;
use core::fmt::Debug;
use core::ops::DerefMut;

//#[derive(Debug)] impl dbug on BBBuffer
pub struct UartDMATransport<'a, T>
where
    T: StaticReadBuffer + StaticWriteBuffer,
{
    device_dma_unit: RefCell<T>,
    internal_message_buffer: RefCell<Fifo<u8>>,  //Copying dma chunks to the internal buffer for now. Looks tricky to avoid this copy but probably doable

	rx_prod: Producer<'a, U64>,
	rx_cons: Consumer<'a, U64>,
	//tx_prod: Producer<'a, U64>,
	//tx_cons: Consumer<'a, U64>,  //just use dma to receive for now

}

// Arbitrarily picking 64 elements which will very likely correspond to a transacton block size.
// Really depends on overall message size. The transaction bock size shouldn't be too big (whre it exceedsmessag size for instance)
// nor too small (in which case there could be frequent interrupts and defeats purpose of trying too imprve performance)

static bbbuffer: BBBuffer<U64> = BBBuffer( ConstBBBuffer::new() );

impl<'a, T> UartDMATransport<'a, T>
where
    T: StaticReadBuffer + StaticWriteBuffer,
{
    // Can't be const until bounds are allowed.
    pub /*const*/ fn new(dma_unit: T) -> Self {

	let (mut rx_p, rx_c) = bbbuffer.try_split().unwrap();

        Self {
            device_dma_unit: RefCell::new(dma_unit),
            internal_message_buffer: RefCell::new(Fifo::new_const()),
			rx_prod: rx_p,
			rx_cons: rx_c,    
        }
    }
}