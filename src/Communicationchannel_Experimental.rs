extern crate futures; // 0.1.23
extern crate rand;
use futures::{sync::mpsc, Async, Sink, Stream};
//use std::sync::mpsc;
use std::thread;
use prost::Message;
use bytes::{BytesMut, BufMut, BigEndian};
use futures::future::{ ok};
use serde::{Serialize, Deserialize};



use std::time::Duration;
use rand::distributions::{Range, IndependentSample};


#[derive(Serialize, Deserialize, Debug)]
enum signal{
  GET_PC,
  SET_PC,
  WRITE_WORD,
  READ_WORD,
  PAUSE,
  SET_BREAKPOINT,
  UNSET_BREAKPOINT,
  GET_BREAKPOINTS,
  GET_MAX_BREAKPOINTS,
  SET_MEMORY_WATCH,
  UNSET_MEMORY_WATCH,
  GET_MEMORY_WATCHES,
  GET_MAX_MEMORY_WATCHES,
  STEP,
  RUN_UNTIL_EVENT,
}


#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
    message: signal,
}
#[derive(Serialize, Deserialize, Debug)]
struct set_pc{
    message: signal,
    addr: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_pc{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct write_word{
    message: signal,
    addr: u16,
    word: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct read_word{
    message: signal,
    addr: u16,
}


// struct write_word{
//     message: String,
//     addr: u16,
//     word: u16,
// }
#[derive(Serialize, Deserialize, Debug)]
struct set_breakpoint{
    message: signal,
    addr: u16,
}

// struct set_breakpoint{
//     message: String,
//     addr: u16,
// }
#[derive(Serialize, Deserialize, Debug)]
struct unset_breakpoint{
    message: signal,
    idx: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_breakpoints{
    message: signal,
    addr: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_max_breakpoints{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct set_memory_watch{
    message: signal,
    addr: u16,
}

// struct set_memory_watch{
//     message: signal,
//     addr: u16,
// }
#[derive(Serialize, Deserialize, Debug)]
struct unset_memory_watch{
    message: signal,
    idx: usize,
}

// struct get_memory_watches{
//     message: String,
// }

// struct get_max_memory_watches{
//     message: String,
// }

// struct unset_memory_watch{
//     message: String,
// }
#[derive(Serialize, Deserialize, Debug)]
struct step{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct pause{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct get_state{
    message: signal,
}

#[derive(Serialize, Deserialize, Debug)]
struct run_until_event{
    message: signal,
}







    // fn get_pc(&self) -> Addr;
    // fn set_pc(&mut self, addr: Addr); // Should be infallible.

    // fn get_register(&self, reg: Reg) -> Word;
    // fn set_register(&mut self, reg: Reg, data: Word); // Should be infallible.

    // fn write_word(&mut self, addr: Addr, word: Word);
    // fn read_word(&self, addr: Addr) -> Word;
    // fn commit_memory(&self) -> Result<(), ()>;

    // fn set_breakpoint(&mut self, addr: Addr) -> Result<usize, ()>;
    // fn unset_breakpoint(&mut self, idx: usize) -> Result<(), ()>;
    // fn get_breakpoints(&self) -> [Option<Addr>; MAX_BREAKPOINTS];


    // fn set_memory_watch(&mut self, addr: Addr) -> Result<usize, ()>;
    // fn unset_memory_watch(&mut self, idx: usize) -> Result<(), ()>;
    // fn get_memory_watches(&self) -> [Option<Addr>; MAX_MEMORY_WATCHES];


    // // Execution control functions:
    // fn run_until_event(&mut self) -> Self::EventFuture; // Can be interrupted by step or pause.
    // fn step(&mut self);
    // fn pause(&mut self);

    // fn get_state(&self) -> State;


        fn get_pc(){
        let point = get_pc { message: signal::GET_PC};
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: get_pc = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }

         fn set_pc(){
        let point = set_pc { message: signal::SET_PC, addr: 2000 };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: set_pc = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }

        fn step(){
        let point = step { message: signal::STEP};
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: set_pc = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }

        fn pause(){
        let point = pause { message: signal::PAUSE };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: pause = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }

        fn set_memory_watch(){
        let point = set_memory_watch { message: signal::SET_MEMORY_WATCH, addr: 2000 };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: set_memory_watch = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }

                fn set_breakpoint(){
        let point = set_breakpoint { message: signal::SET_BREAKPOINT, addr: 2000 };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: set_breakpoint = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }


        fn unset_breakpoint(){
        let point = unset_breakpoint { message: signal::UNSET_BREAKPOINT, idx: 5 };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: unset_breakpoint = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }


             fn read_word(){
        let point = read_word { message: signal::READ_WORD, addr: 2000};
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: read_word = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }


                fn unset_memory_watch(){
        let point = unset_memory_watch { message: signal::UNSET_MEMORY_WATCH, idx: 2000 };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: unset_memory_watch = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }



                fn write_word(){
        let point = write_word { message: signal::WRITE_WORD, addr: 2000, word: 1000 };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);
        let deserialized: write_word = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message: {}", receiver.recv().unwrap());
        }


        fn run_until_event( ){
        let point = run_until_event { message: signal::RUN_UNTIL_EVENT };
        let serialized = serde_json::to_string(&point).unwrap();
        println!("serialized = {}", serialized);

        let mut buf = BytesMut::with_capacity(1024);
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(serialized).unwrap();
        println!("got message:");
        let deserialized: run_until_event = serde_json::from_str(&receiver.recv().unwrap()).unwrap();
        println!("deserialized = {:?}", deserialized);
         

        }

        // fn test_device_reception(){
        //     let (mut tx, rx) = mpsc::channel(1024);
        //         let handle = thread::spawn(move || {
        //     tx.send(1)
        //         .and_then(|tx| tx.send(2))
        //         .and_then(|tx| tx.send(3))
        //         .wait()
        //         .expect("Unable to send");
        // });

        // let mut rx = rx.map(|x| x * x);

        // handle.join().unwrap();

        // while let Ok(Async::Ready(Some(v))) = rx.poll() {
        //     println!("stream: {}", v);
        // }

        // }




// fn main() {
// 	    //let point = Point {message: signal::SET_PC, addr: 2000 };

//     // Convert the Point to a JSON string.
//     //let serialized = serde_json::to_string(&point).unwrap();

//     // Prints serialized = {"x":1,"y":2}
//     //println!("serialized = {}", serialized);

//     // Convert the JSON string back to a Point.
//     //let deserialized: Point = serde_json::from_str(&serialized).unwrap();

//     // Prints deserialized = Point { x: 1, y: 2 }
//     //println!("deserialized = {:?}", deserialized);
//     get_pc();
//     set_pc();
//     pause();
//     set_memory_watch();
//     unset_memory_watch();
//     write_word();
//     unset_breakpoint();
//     set_breakpoint();
//     read_word();
//     run_until_event();
//     test_device_reception();
//   //  let (tx, rx) = mpsc::channel(1000);
//       let temp:i32 = 1;
//   //  let mut xs: [i32; 5] = [1, 2, 3, 4, 5];



//     //println!("{}", a);
//   let (mut tx, rx) = mpsc::channel(1024);

//     thread::spawn(move || {
//         println!("--> START THREAD");
//         // We'll have the stream produce a series of values.
//         for _ in 0..10 {

//             let waited_for = sleep_temp();
//             println!("+++ THREAD WAITED {}", waited_for);

//             // When we `send()` a value it consumes the sender. Returning
//             // a 'new' sender which we have to handle. In this case we just
//             // re-assign.
//             match tx.send(waited_for).wait() {
//                 // Why do we need to do this? This is how back pressure is implemented.
//                 // When the buffer is full `wait()` will block.
//                 Ok(new_tx) => tx = new_tx,
//                 Err(_) => panic!("Oh no!"),
//             }

//         }
//         println!("<-- END THREAD");
//         // Here the stream is dropped.
//     });

//     // We can `.fold()` like we would an iterator. In fact we can do many
//     // things like we would an iterator.
//     let sum = rx.fold(0, |acc, val| {
//             // Notice when we run that this is happening after each item of
//             // the stream resolves, like an iterator.
//             println!("+++ FOLDING {} INTO {}", val, acc);
//             // `ok()` is a simple way to say "Yes this worked."
//             // `err()` also exists.
//             ok(acc + val)
//         })
//         .wait()
//         .unwrap();
//     println!("SUM {}", sum);
// }

use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut; 
thread_local!(static NOTIFY: RefCell<bool> = RefCell::new(true));

struct Context<'a> {
    waker: &'a Waker,
}

impl<'a> Context<'a> {
    fn from_waker(waker: &'a Waker) -> Self {
        Context { waker }
    }

    fn waker(&self) -> &'a Waker {
        &self.waker
    }
}

struct Waker;

#[derive(Serialize, Deserialize, Debug)]
enum device_status{
    PAUSE_SUCCESSFUL,
    STEP_SUCCESSFUL,
    RUN_COMPLETED,
    RUN_FAILED,
    STEP_UNSUCCESSFUL,
    PAUSE_UNSUCCESSFUL,

}
impl Waker {
    fn wake(&self) {
        NOTIFY.with(|f| *f.borrow_mut() = true)
    }
}

enum Poll<T> {
    Ready(T),
    Pending,
}

trait Future {
    type Output;

    fn poll(&mut self, cx: &Context) -> Poll<Self::Output>;
}

#[derive(Default)]
struct Device_Signal {
    count: u32,
}
impl Deref for Device_Signal{
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.count
    }
}

impl DerefMut for Device_Signal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.count
    }
}

impl Future for Device_Signal {
    type Output = device_status;

    fn poll(&mut self, ctx: &Context) -> Poll<Self::Output> {
        match self.count {
           // let ret_status = device_status::PAUSE_SUCCESSFUL;

            1 => Poll::Ready(device_status::RUN_COMPLETED),
            2 => Poll::Ready(device_status::STEP_SUCCESSFUL),
            3 => Poll::Ready(device_status::PAUSE_SUCCESSFUL),
            4 => Poll::Ready(device_status::RUN_FAILED),
            5 => Poll::Ready(device_status::PAUSE_UNSUCCESSFUL),
            6 => Poll::Ready(device_status::STEP_UNSUCCESSFUL),
            _ => {
                ctx.waker().wake();
                Poll::Pending
            }
        }
    }
}

fn run<F>(mut f: F) -> F::Output
where
    F: Future,
{
    NOTIFY.with(|n| loop {
        if *n.borrow() {
            *n.borrow_mut() = false;
            let ctx = Context::from_waker(&Waker);
            if let Poll::Ready(val) = f.poll(&ctx) {
                return val;
            }
        }
    })
}

// fn main() {
//    // let (sender, receiver) = std::sync::mpsc::channel();
//     let mut my_future = Device_Signal::default();

//     my_future.count = 4;
//     println!("Output: {:?}", run(my_future));
//     println!("Execution unblocked");
// }


static NTHREADS: usize = 1;
use std::sync::{Arc, Mutex};
fn main() {
    // Channels have two endpoints: the `Sender<T>` and the `Receiver<T>`,
    // where `T` is the type of the message to be transferred
    // (type annotation is superfluous)
    use std::sync::mpsc::{Sender, Receiver};
    let (tx, rx): (Sender<usize>, Receiver<usize>) = std::sync::mpsc::channel();
    let (tx2, rx2): (Sender<usize>, Receiver<usize>) = std::sync::mpsc::channel();
   // let mut ids2 = Vec::with_capacity(NTHREADS);

        // The sender endpoint can be copied
        let thread_tx = tx.clone();
    //let mut my_future = Arc::new((Device_Signal::default()));
    let mut my_future = std::sync::Arc::new(std::sync::Mutex::new(Device_Signal::default()));
   // let mutex = std::sync::Mutex::new(foo);
//let arc = std::sync::Arc::new(mutex);

   // my_future.count = 8;
        // Each thread will send its id via the channel
        thread::spawn(move || {
            // The thread takes ownership over `thread_tx`
            // Each thread queues a message in the channel
           // thread_tx.send(3).unwrap();
         //  Arc::downgrade(&my_future);
          // (my_future).count=4;
          let mut guard = my_future.lock().unwrap();
          guard.count = 4;
            rx2.recv();
             tx.send(1);
            // Sending is a non-blocking operation, the thread will continue
            // immediately after sending its message
           // println!("thread {} finished", id);
            //ids2.push(rx2.recv());
        });
        // thread::spawn(move ||{

        
        // } );
    //ISSUE: How is run_future now dereferenced
   // println!("Output: {:?}", run(my_future));
    tx2.send(2);
    (rx.recv());
        
    

    // Here, all the messages are collected
  //  let mut ids = Vec::with_capacity(NTHREADS);
 
        // The `recv` method picks a message from the channel
        // `recv` will block the current thread if there no messages available
 

 

    // Show the order in which the messages were sent
    //println!("{:?}", ids);
}
pub fn sleep_temp() -> u64 {
    let mut generator = rand::thread_rng();
    let possibilities = Range::new(0, 100);

    let choice = possibilities.ind_sample(&mut generator);

    let a_little_bit = Duration::from_millis(choice);
    thread::sleep(a_little_bit);
    choice
}

