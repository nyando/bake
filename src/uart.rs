/// uart.rs - UART communication for Bali

extern crate serialport;
use serialport::{SerialPort};

fn open() {

    ///
    /// open a UART connection to COM or TTY port for the FPGA
    /// configuration:
    ///  - baudrate: 9600
    ///  - data bits: 8
    ///  - parity bit: none
    ///  - flow control: none
    ///
    /// handshake? (would need to be implemented in SV too)
    ///

}

fn write(mem: Vec<u8>) {

    /// send all bytes in mem

}