//! This "attempts" to use DMA (device impl can)and does not attempt to be zero-copy.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};
//use lc3_tm4c::peripherals_generic::dma::DmaChannel;

 use embedded_dma::{StaticReadBuffer, StaticWriteBuffer};
 use bbqueue::{BBBuffer, GrantR, GrantW, ConstBBBuffer, Consumer, Producer, ArrayLength, consts::*};
 use nb::block;

use core::cell::RefCell;
use core::fmt::Debug;
use core::ops::DerefMut;
use core::ops::Deref;
use stable_deref_trait::StableDeref;

pub struct Dma1Channel <Peripheral> {
    peripheral: Peripheral
}

//A trait for  a dma channel. A physical peripheral
pub trait DmaChannel {

    //Device secific preinitialization to enable DMA
    fn dma_device_init(&self);

    /// Data will be written to this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_destination_address(&self, address: usize);

    /// Data will be read from this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_source_address(&self, address: usize);

    /// Number of bytes to transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_transfer_length(&self, len: usize);

    /// Starts the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_start(&self);

    /// Stops the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_stop(&mut self);

    /// Returns `true` if there's a transfer in progress
    ///
    /// NOTE this performs a volatile read
    fn dma_in_progress(&mut self) -> bool;

    fn dma_num_bytes_transferred(&mut self) -> usize;
}

pub struct UartDMATransport<'a, T>
where
	T: DmaChannel
{
    device_dma_unit: RefCell<T>,
    internal_message_buffer: RefCell<Fifo<u8>>,

	rx_prod: RefCell<Producer<'a, U64>>,
	rx_cons: RefCell<Consumer<'a, U64>>,
	tx_prod: RefCell<Producer<'a, U64>>,
	tx_cons: RefCell<Consumer<'a, U64>>, 

}


pub struct bbbuffer{
	message_buffer: BBBuffer<U2048>
}


static rx_buffer: BBBuffer<U64> = BBBuffer( ConstBBBuffer::new() );
static tx_buffer: BBBuffer<U64> = BBBuffer( ConstBBBuffer::new() );

//unsafe impl StableDeref for bbbuffer {}

// impl Deref for bbbuffer {
// // where
// //	N: ArrayLength<u8>,

//     type Target = u32;//BBBuffer<U2048>;

//     fn deref(&self) -> &Self::Target {
//         unimplemented!()
//         //&self.message_buffer
//     }
// }

//  unsafe impl StaticReadBuffer for bbbuffer
//  {
// // // where
// // // 	N: DmaChannel,
//     type Word = u32;
    
//     fn static_read_buffer(&self) -> (*const <Self as StaticReadBuffer>::Word, usize){
//         unimplemented!()
//     }
//  }


impl<'a, T> UartDMATransport<'a, T>
where
    T: DmaChannel,
{
    pub fn new(dma_unit: T) -> Self {

		let (rx_p, rx_c) = rx_buffer.try_split().unwrap();
        let (tx_p, tx_c) = tx_buffer.try_split().unwrap();

        Self {
            device_dma_unit: RefCell::new(dma_unit),
            internal_message_buffer: RefCell::new(Fifo::new_const()),
			rx_prod: RefCell::new(rx_p),
			rx_cons: RefCell::new(rx_c),
			tx_prod: RefCell::new(tx_p),
			tx_cons: RefCell::new(tx_c),
        }
    }
}


impl<'a, T> Transport<Fifo<u8>, Fifo<u8>> for UartDMATransport<'a, T>
where
	T: DmaChannel
{
    type RecvErr = ();
    type SendErr = ();

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("simp");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    fn send(&self, message: Fifo<u8>) -> Result<(), ()>  {
		unimplemented!()
    }

    fn get(&self) -> Result<Fifo<u8>, Option<Self::RecvErr>> {  // keeping the fifo for now to test with the xisting device impl. TODO: use slice directly in poll impl to avoid repeated copies

        let mut buf = self.internal_message_buffer.borrow_mut();
        let mut cons_buf = self.rx_cons.borrow_mut();
        let mut device_dma_unit = self.device_dma_unit.borrow_mut();

        let mut ret: Result<Fifo<u8>, Option<Self::RecvErr>> = Err(None);

        let mut rx_grant = self.rx_prod.borrow_mut().grant_exact(50).unwrap();
    	device_dma_unit.dma_set_destination_address(rx_grant.buf().as_ptr() as usize);
    	device_dma_unit.dma_set_transfer_length(50);   // change this 50 (ballpark message size to max message size;)
    	device_dma_unit.dma_start();

            if(device_dma_unit.dma_num_bytes_transferred() > 0){
	            rx_grant.commit(device_dma_unit.dma_num_bytes_transferred());
	            let mut rx_buf = cons_buf.read().unwrap();
	            for i in 0..device_dma_unit.dma_num_bytes_transferred() {
	            	buf.push(rx_buf[i]).unwrap();
	            }

	            if(rx_buf[device_dma_unit.dma_num_bytes_transferred() - 1] == 0){
	            	ret = Ok(core::mem::replace(&mut buf, Fifo::new()))
	            }

	            rx_buf.release(device_dma_unit.dma_num_bytes_transferred());

	        }

	        ret
    }
}