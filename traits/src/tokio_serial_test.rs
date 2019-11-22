//extern crate tokio_serial;
use crate::transport_layer::TransportLayer;
use crate::transport_layer::Message;
extern crate bytes;
//extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serial;
extern crate serde;
extern crate serde_json;
use std::io::Write;
use std::{env, io, str};
use tokio_io::codec::{Decoder, Encoder};
//use std::{env, io, str};

use bytes::BytesMut;
struct TokioTransportLayer{
	tty_path: &'static str,
	settings: tokio_serial::SerialPortSettings,
	port:     tokio_serial::Serial,



}
impl TokioTransportLayer{
	fn initialize(&mut self){
		self.tty_path="COM3";
		self.settings=tokio_serial::SerialPortSettings::default();
		self.port = tokio_serial::Serial::from_path(self.tty_path, &self.settings).unwrap();


	}
}

impl TransportLayer for TokioTransportLayer {
	//fn initialize(){}
   fn send(&self, message: Message) -> Result<(), ()>{
   	    let point = message;
        let serialized = serde_json::to_string(&point).unwrap();
       // self.port.write(serialized);
   	Ok(())

   }
   
   fn get(&self) -> Option<Message>{
   	Some(Message::RUN_UNTIL_EVENT)
   }
}