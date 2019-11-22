fn main() {






    
    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
   
   // let (tx, rx): (Sender<std::string::String>, Receiver<std::string::String>) = std::sync::mpsc::channel();
   // let (tx2, rx2): (Sender<device_status>, Receiver<device_status>) = std::sync::mpsc::channel();
   // let mut ids2 = Vec::with_capacity(NTHREADS);
   let mut overall_channel = message_channel{
    channel1: std::sync::mpsc::channel(),
    channel2: std::sync::mpsc::channel(),
   };
   let (tx2, rx2) = overall_channel.channel2;
   let (tx, rx)   = overall_channel.channel1;

   let mut host_cpy = host_control{
    host_to_device_transmitter: tx,
    device_to_host_receiver:    rx2,
   };
}

