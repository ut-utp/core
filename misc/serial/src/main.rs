
use tokio_serial::{Serial, SerialPortSettings, DataBits, FlowControl, Parity, StopBits};
use tokio::io::AsyncReadExt;

use std::time::Duration;

const BAUD_RATE: u32 = 1_500_000;
// const BAUD_RATE: u32 = 15200;
// const BAUD_RATE: u32 = 115_200;
// const BAUD_RATE: u32 = 2_300_000;
// const BAUD_RATE: u32 = 2_500_000;
// const BAUD_RATE: u32 = 2_500_000;

// #[tokio::main(basic_scheduler)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = SerialPortSettings {
        baud_rate: BAUD_RATE,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        // parity: Parity::Even,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_secs(100),
    };

    let mut dev = Serial::from_path("/dev/lm4f", &settings)?;

    loop {
        print!("{}", dev.read_u8().await? as char);
    }

    Ok(())
}
