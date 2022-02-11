mod structs;
use structs::*;

mod opcodes;
use opcodes::*;

use std::env;

const READ : &str = "read";

fn print_method(name : &str, code_info : &BaliCode) {

    let opmap = op_map();
    let mut code_iter = code_info.code.iter();
    
    println!("method {} {}, stack size {}, locals: {}", name, code_info.signature, code_info.max_stack, code_info.max_locals);
    
    while let Some(opcode) = code_iter.next() {
        
        let op : &Op = &opmap[opcode];
        
        match op.args {
            
            0 => println!("{}", op.mnemonic),
            
            1 => {
                let arg = code_iter.next().unwrap();
                println!("{}: {:#04x}", op.mnemonic, arg);
            },
            
            2 => {
                let arg1 = code_iter.next().unwrap();
                let arg2 = code_iter.next().unwrap();
                println!("{}: {:#06x}", op.mnemonic, ((*arg1 as u16) << 8 | (*arg2 as u16)));
            },
            
            _ => println!("unknown opcode")
        }
    }

}

fn analyze(classinfo : &ClassFile) {
    
    for (name, code_info) in code_blocks(&classinfo) {
        print_method(&name, &code_info);
    }

}

fn main() {

    let args : Vec<String> = env::args().collect();

    let task : &str = &args[1].to_lowercase();
    let filepath : &str = &args[2];
    let classinfo = read_classfile(filepath).unwrap();

    match task {
        READ => analyze(&classinfo),
        _    => ()
    };

}
