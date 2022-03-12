extern crate serialport;
use serialport::{SerialPort, DataBits, FlowControl, Parity, StopBits};

use std::time::Duration;

pub fn open_serial(port_id : &str) -> Box<dyn SerialPort> {
    serialport::new(port_id, 9600)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("could not open serial port")
}

pub fn binwrite(port : &mut Box<dyn SerialPort>, bin : &[u8]) -> std::io::Result<()> {
    for byte in bin {
        
        let to_write : Vec<u8> = vec!(*byte);
        port.write_all(&to_write)?;

        let mut response : Vec<u8> = vec![0; 1];
        port.read_exact(&mut response)?;
        for entry in response { println!("serial port confirmed byte {:#04x}", entry); }

    }
    
    Ok(())
}

pub fn _memread(_port : &mut Box<dyn SerialPort>, _lo : usize, _hi : usize) {
    // read from Bali processor memory
}