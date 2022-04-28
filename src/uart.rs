use serialport::{SerialPort, DataBits, FlowControl, Parity, StopBits};

use std::time::Duration;

pub fn open_serial(port_id : &str) -> Box<dyn SerialPort> {
    serialport::new(port_id, 9600)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(10000))
        .open()
        .expect("could not open serial port")
}

pub fn binwrite(port : &mut Box<dyn SerialPort>, bin : &[u8], long : bool) -> Result<u64, std::io::Error> {

    if !long {
        
        let memlen : Vec<u8> = vec!(bin.len().try_into().unwrap());
        port.write_all(&memlen)?;
        let mut response : Vec<u8> = vec![0; 1];
        port.read_exact(&mut response)?;
    
    } else {
        
        let binlen : u16 = bin.len().try_into().unwrap();
        
        let mut memlen : Vec<u8> = Vec::<u8>::new();
        memlen.push((binlen & 0xff) as u8);
        memlen.push((binlen >> 8) as u8);

        for byte in memlen {
            port.write_all(&[byte])?;
            let mut response : Vec<u8> = vec![0; 1];
            port.read_exact(&mut response)?;
        }

    }
    
    for byte in bin {
        
        let to_write : Vec<u8> = vec!(*byte);
        port.write_all(&to_write)?;

        let mut response : Vec<u8> = vec![0; 1];
        port.read_exact(&mut response)?;
        
    }

    let mut cycles : Vec<u8> = vec![0; 8];
    port.read_exact(&mut cycles)?;
    
    let turnaround : u64 = (( cycles[7] as u64 ) << 56) + 
                           (( cycles[6] as u64 ) << 48) + 
                           (( cycles[5] as u64 ) << 40) + 
                           (( cycles[4] as u64 ) << 32) + 
                           (( cycles[3] as u64 ) << 24) + 
                           (( cycles[2] as u64 ) << 16) + 
                           (( cycles[1] as u64 ) << 8)  + 
                            ( cycles[0] as u64 );
    
    Ok(turnaround)
}
