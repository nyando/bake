/// uart.rs - UART communication for Bali

extern crate serialport;
use serialport::{DataBits, FlowControl, Parity, StopBits};

use std::time::Duration;

pub fn open_serial(port_id : &str) -> std::io::Result<()> {
    let mut port = serialport::new(port_id, 9600)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("could not open serial port");

    for i in 0..255 {
        let input : Vec<u8> = vec!(i);

        port.write_all(&input)?;
        println!("wrote {:#04x} to serial port", i);
        
        let mut buffer : Vec<u8> = vec![0; 1];
        port.read(&mut buffer)?;

        for byte in buffer {
            println!("received value {:#04x} on serial port", byte);
        }

    }

    Ok(())
}