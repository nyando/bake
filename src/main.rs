mod structs;
use structs::*;

mod opcodes;
use opcodes::*;

use std::env;

fn main() {
    let args : Vec<String> = env::args().collect();
    let classinfo = read_classfile(&args[1]).unwrap();
    let opmap = op_map();
    
    for (opcode, op) in op_map() {
        println!("Opcode: {}, Mnemonic: {}, Args: {}", opcode, op.mnemonic, op.args);
    }

    for (index, value) in utf8_constants(&classinfo) {
        println!("Index: {}, Value: {}", index, value);
    }

    for (name, code) in code_blocks(&classinfo) {
        println!("Method {}", name);

        let mut code_iter = code.iter();
        
        while let Some(opcode) = code_iter.next() {
            let op : &Op = &opmap[opcode];
            match op.args {
                0 => println!("{}", op.mnemonic),
                1 => println!("{}: {}", op.mnemonic, code_iter.next().unwrap()),
                2 => println!("{}: {} {}", op.mnemonic, code_iter.next().unwrap(), code_iter.next().unwrap()),
                _ => println!("unknown opcode")
            }
        }
    }
}
