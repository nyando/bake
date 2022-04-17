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

pub fn binwrite(port : &mut Box<dyn SerialPort>, bin : &[u8], long : bool) -> Result<u32, std::io::Error> {

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
            port.write_all(&vec!(byte))?;
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

    let mut cycles : Vec<u8> = vec![0; 4];
    port.read_exact(&mut cycles)?;
    
    let turnaround : u32 = (( cycles[3] as u32 ) << 24) + 
                           (( cycles[2] as u32 ) << 16) + 
                           (( cycles[1] as u32 ) << 8)  + 
                            ( cycles[0] as u32 );
    
    Ok(turnaround)
}
