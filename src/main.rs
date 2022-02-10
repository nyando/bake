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
        println!("opcode {}, mnemonic {}, args {}", opcode, op.mnemonic, op.args);
    }

    for (index, value) in utf8_constants(&classinfo) {
        println!("index {}, value {}", index, value);
    }

    for (name, code_info) in code_blocks(&classinfo) {
        println!("method {}, stack size {}, locals: {}", name, code_info.max_stack, code_info.max_locals);

        let mut code_iter = code_info.code.iter();
        
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
