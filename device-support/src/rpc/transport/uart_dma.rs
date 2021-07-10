//! This "attempts" to use DMA (device impl can)and does not attempt to be zero-copy.

use crate::util::Fifo;

use lc3_traits::control::rpc::Transport;
use lc3_traits::control::{Identifier, Version, version_from_crate};
//use lc3_tm4c::peripherals_generic::dma::DmaChannel;

use embedded_hal::serial::{Read, Write};
use bbqueue::{BBBuffer, GrantR, GrantW, ConstBBBuffer, Consumer, Producer, ArrayLength, consts::*};
use nb::block;

use core::cell::RefCell;
use core::fmt::Debug;
use core::ops::DerefMut;
use core::ops::Deref;

pub struct Dma1Channel <Peripheral> {
    peripheral: Peripheral
}

//A trait for  a dma channel. A physical peripheral
pub trait DmaChannel {

    //Device secific preinitialization to enable DMA
    fn dma_device_init(&mut self);

    /// Data will be written to this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_destination_address(&mut self, address: usize);

    /// Data will be read from this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_source_address(&mut self, address: usize);

    /// Number of bytes to transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_set_transfer_length(&mut self, len: usize);

    /// Starts the DMA transfer
    ///
    /// NOTE this performs a volatile write
    fn dma_start(&mut self);

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

pub struct UartDMATransport<'a, T, W: Write<u8>>
where
	T: DmaChannel,
    <W as Write<u8>>::Error: Debug,
{
    device_dma_unit: RefCell<T>,
    write: RefCell<W>,

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


impl<'a, T, W:Write<u8>> UartDMATransport<'a, T, W>
where
    T: DmaChannel,
    <W as Write<u8>>::Error: Debug,
{
    pub fn new(dma_unit: T, write: W) -> Self {

		let (rx_p, rx_c) = rx_buffer.try_split().unwrap();
        let (tx_p, tx_c) = tx_buffer.try_split().unwrap();

        Self {
            device_dma_unit: RefCell::new(dma_unit),
            write: RefCell::new(write),
            internal_message_buffer: RefCell::new(Fifo::new_const()),
			rx_prod: RefCell::new(rx_p),
			rx_cons: RefCell::new(rx_c),
			tx_prod: RefCell::new(tx_p),
			tx_cons: RefCell::new(tx_c),
        }
    }
}


impl<'a, T, W:Write<u8>> Transport<Fifo<u8>, Fifo<u8>> for UartDMATransport<'a, T, W>
where
	T: DmaChannel,
    <W as Write<u8>>::Error: Debug,
{
    type RecvErr = ();
    type SendErr = W::Error;

    const ID: Identifier = Identifier::new_from_str_that_crashes_on_invalid_inputs("UART");
    const VER: Version = {
        let ver = version_from_crate!();

        let id = Identifier::new_from_str_that_crashes_on_invalid_inputs("simp");

        Version::new(ver.major, ver.minor, ver.patch, Some(id))
    };

    //TODO: Use DM here too
    fn send(&self, message: Fifo<u8>)  -> Result<(), W::Error>{
        let mut write = self.write.borrow_mut();

        for byte in message {
            block!(write.write(byte))?
        }

        block!(write.flush())
    }

    fn get(&self) -> Result<Fifo<u8>, Option<Self::RecvErr>> {  // keeping the fifo for now to test with the xisting device impl. TODO: use slice directly in poll impl to avoid repeated copies

        let mut buf = self.internal_message_buffer.borrow_mut();
        let mut cons_buf = self.rx_cons.borrow_mut();
        let mut device_dma_unit = self.device_dma_unit.borrow_mut();

        let mut ret: Result<Fifo<u8>, Option<Self::RecvErr>> = Err(None);

        let mut rx_grant = self.rx_prod.borrow_mut().grant_exact(50).unwrap();
    	device_dma_unit.dma_set_destination_address(rx_grant.buf().as_ptr() as usize);

    	rx_grant.commit(50);    //This relies on the contiguous buffer property. Commiting and having the receive buffer ready to read before the data is even ready
    						 	//is probably not how bbqueue is meant to be use. But I see no other way to do it because commiting "consumes" the grant and you can't use it again
    						 	//so you can't commit again without reinitializing. Commiting dma_num_bytes_transferred() and read as data arrives in the loop would have been a nicer way to do it but not possibe because of this. Any better way?
    						 	// The way bbqueue is used right now makes it seem pretty useless. Might as well just use a simple buffer/Fifo. bbqueue however would be ideal for fixed message sizes.
    	let mut rx_buf = cons_buf.read().unwrap(); 

    	device_dma_unit.dma_set_transfer_length(50);   // change this 50 (ballpark message size to max message size;)
    	device_dma_unit.dma_start();

    	loop{

    		let bytes_transferred = device_dma_unit.dma_num_bytes_transferred(); //asynchronous number of bytes so call this once and use that vlue through loop to process


            if(bytes_transferred > 0){
	            
	            for i in 0..bytes_transferred {    //get rid of this after changing device step impl to use slice/simpler common data structure
	            	buf.push(rx_buf[i]).unwrap();
	            }

	            if(rx_buf[bytes_transferred - 1] == 0){
	            	rx_buf.release(50);
	            	break Ok(core::mem::replace(&mut buf, Fifo::new()))
	            }

	        }
	        else{
	        	rx_buf.release(50); //DMA should be pretty fast so that this case doesn't happen if data is actually coming in (atleast some data will be available within a loop iteration). So under normal circumstances this should
	        						//only happen when no data is coming in (just bare event loop running)
	        	break Err(None)  //should be nn blocking
	        }

	    }
    }
}