//! TODO!

enum State {
    WaitingForMessageType,
    ControlMessageLengthUnknown,
    ConsoleInputLengthUnknown,
    ReadingControlMessage(u16),
    ReadingConsoleInput(u16),
}

static RX: Mutex<RefCell<Rx<_>>> = ..;
static RX_STATE: Mutex<RefCell<State>> = Mutex::new(RefCell::new(State::WaitingForMessageType));
static RX_BUFFER: Mutex<RefCell<Fifo>> = Mutex::new(RefCell::new(Fifo::<u8>::new()));

static CONTROL_MESSAGE_PENDING: Mutex<Cell<Option<ControlMessage>>> = Mutex::new(Cell::new(None)); // TODO: should this be more than one element big?
static CONTROL_MESSAGE_FLAG: AtomicBool = AtomicBool::new(false);

const CONTROL_MESSAGE_SLUG: u8 = 0b1000_0000;
const CONSOLE_INPUT_SLUG: u8 = 0b1100_0000;

// TODO: invoked on any new data or FIFO half full?
// any new data for now
#[interrupt]
fn uart_rx_handler() {
    interrupt_free(|cs| {
        let rx_guard = RX.lock(cs);
        let rx = rx_guard.borrow_mut();

        let rx_state_guard = RX_STATE.lock(cs);
        let rx_state = rx_state_guard.borrow_mut();

        let rx_buf_guard = RX_BUFFER.lock(cs);
        let rx_buf = rx_state_guard.borrow_mut();

        use State::*;

        while let Ok(c) = rx.read() {
            rx_state = match (rx_state, c) {
                (WaitingForMessageType, CONTROL_MESSAGE_SLUG) => ControlMessageLengthUnknown,
                (WaitingForMessageType, CONSOLE_INPUT_SLUG) => ConsoleInputLengthUnknown,
                (WaitingForMessageType, _) => panic!("unknown message type"), // TODO: how to actually handle?

                (ConsoleInputLengthUnknown, c) | (ControlMessageLengthUnknown, c) => {
                    rx_buf.push(c).unwrap(); // TODO: don't unwrap here...
                    if let Some(len) = prost::decode_length_delimiter(&mut rx_buf) { // TODO: will this behave correctly with a ref or do we need to extract and call decode_varint?
                        assert!(len <= Fifo::CAPACITY);

                        match rx_state {
                            ConsoleInputLengthUnknown => ReadingConsoleInput(len),
                            ControlMessageLengthUnknown => ReadingControlMessage(len),
                            _ => unreachable!(),
                        }
                    } else {
                        rx_state // Keep reading bytes...
                    }
                },

                (ReadingConsoleInput(1), c) => {
                    rx_buf.push(c).unwrap(); // TODO: don't unwrap...

                    // TODO! actually use the input by feeding it to the input peripheral!
                },

                (ReadingControlMessage(1), c) => {
                    rx_buf.push(c).unwrap(); // TODO: don't unwrap...

                    let m = ControlMessage::decode(&mut rx_buf).unwrap();
                    assert!(rx_buf.length() == 0);

                    let cm = CONTROL_MESSAGE_PENDING.lock(cs);
                    assert_eq!(None, cm.replace(Some(m)));

                    assert_eq!(CONTROL_MESSAGE_FLAG.load(Ordering::SeqCst), false);
                    CONTROL_MESSAGE_FLAG.store(true, Ordering::SeqCst);
                },
            }
        }

        // rx_state = match rx_state {
        //     WaitingForMessageType => {
        //         let ty: u8 = 0;
        //         rx.read(&mut ty).expect("at least one new character...");

        //         match ty {
        //             CONTROL_MESSAGE_SLUG =>
        //         }
        //     }
        // }
    })

    // TODO: acknowledge interrupt?
}

struct UartTransport {
    tx: Tx<_> // TODO
}

impl TransportLayer for UartTransport {
    fn get_message(&mut self) -> Option<ControlMessage> {
        if CONTROL_MESSAGE_FLAG.load(Ordering::SeqCst) {
            let cm = CONTROL_MESSAGE_PENDING.lock();
            let m = cm.take();

            // assert!(m.is_some()); // This invariant should be maintained, but w/e.
            m
        } else {
            None
        }
    }

    fn send_message(&mut self, message: ControlMessage) -> Result<(), ()> {
        let m_len = message.encoded_len();
        let len_len = prost::length_delimiter_len(len);

        let len = m_len + len_len;
        assert!(len <= TX_BUF_CAPACITY);

        const TX_BUF_CAPACITY: usize = 256; // TODO: again, compile time checks, etc.
        let mut buf: [u8; TX_BUF_CAPACITY] = [0; TX_BUF_CAPACITY];

        message.encode_length_delimited(&mut buf).unwrap(); // TODO: don't unwrap...

        // nb::block!(tx.)
        // TODO: maybe use DMA instead of using a blocking write here..
        tx.write(CONTROL_MESSAGE_SLUG);
        tx.write_all(buf[0..len]);
    }
}

fn main() {
    let mut sim = <snipped>;
    let mut control_client: Client<UartTransport> = <snipped>; // got to create the UART pair, split it, give the Tx to a UartTransport, and then give that UartTransport to a Client.

    // Actually we need shared access to a Tx since the Output peripheral will need access to it too.
    // So I guess UartTransport holds no state and we'll need more Mutex<RefCell<_>>s!
    // (TODO)

    loop {
        // Do simulator things, etc, etc.

        // if CONTROL_MESSAGE_FLAG.load(Ordering::SeqCst) {
        //     let cm
        // }

        control_client.step(&mut sim);
    }
}
