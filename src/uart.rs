use serialport::{SerialPort, DataBits, FlowControl, Parity, StopBits};

use std::time::Duration;

pub fn open_serial(port_id : &str) -> Box<dyn SerialPort> {
    serialport::new(port_id, 9600)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("could not open serial port")
}

pub fn binwrite(port : &mut Box<dyn SerialPort>, bin : &[u8]) -> std::io::Result<()> {

    let memlen : Vec<u8> = vec!(bin.len().try_into().unwrap());
    port.write_all(&memlen)?;
    let mut response : Vec<u8> = vec![0; 1];
    port.read_exact(&mut response)?;
    for entry in response { println!("program length {:#04x} confirmed", entry); }

    for byte in bin {
        
        let to_write : Vec<u8> = vec!(*byte);
        port.write_all(&to_write)?;

        let mut response : Vec<u8> = vec![0; 1];
        port.read_exact(&mut response)?;
        for entry in response { println!("serial port confirmed byte {:#04x}", entry); }

    }

    println!("finished writing program, waiting for turnaround time response");

    let mut cycles : Vec<u8> = vec![0; 4];
    port.read_exact(&mut cycles)?;
    for entry in cycles { println!("serial port confirmed byte {:#04x}", entry); }
    
    Ok(())
}

pub fn readcycles(port : &mut Box<dyn SerialPort>) -> std::io::Result<()> {

    let mut response : Vec<u8> = vec![0; 4];
    port.read_exact(&mut response)?;
    for entry in response { println!("cycle counter value {:#04x}", entry); }

    Ok(())

}

pub fn _memread(_port : &mut Box<dyn SerialPort>, _lo : usize, _hi : usize) {
    // read from Bali processor memory
}