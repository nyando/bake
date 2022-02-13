mod structs;
use structs::*;

mod opcodes;
use opcodes::*;

use std::env;

const CONST : &str = "consts";
const METHOD : &str = "methods";

fn print_consts(classinfo : &ClassFile) {
    let constpool = constants(&classinfo);

    for (index, value) in &constpool {
        match value {
            ConstPoolValue::Class(name_ref) => println!("{}: {}", index, name_ref),
            ConstPoolValue::Integer(int_const) => println!("{}: {}", index, int_const),
            ConstPoolValue::MethodRef(_, desc_ref) => println!("{}: {}", index, parse_method_signature(&classinfo, desc_ref).unwrap()),
            ConstPoolValue::NameAndType(name_ref, type_ref) => println!("{}: {}, {}", index, name_ref, type_ref),
            ConstPoolValue::UTF8String(str_const) => println!("{}: {}", index, str_const)
        };
    }
}

fn print_method(classinfo : &ClassFile, name : &str, code_info : &BaliCode) {

    let opmap = opmap();
    let mut code_iter = code_info.code.iter();
    let methodrefs = methodrefs(&classinfo);
    
    println!("method {}, stack size {}, locals: {}", name, code_info.max_stack, code_info.max_locals);
    
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
                let arg : u16 = (*arg1 as u16) << 8 | (*arg2 as u16);

                // if static method invocation, print signature of invoked method
                if op.mnemonic == "invokestatic" {
                    println!("{}: {}", op.mnemonic, methodrefs[&arg]);
                } else {
                    println!("{}: {:#06x}", op.mnemonic, arg);
                }
            },
            
            _ => println!("unknown opcode")
        }
    }

}

fn main() {

    let args : Vec<String> = env::args().collect();

    let task : &str = &args[1].to_lowercase();
    let filepath : &str = &args[2];
    let classinfo = read_classfile(filepath).unwrap();

    match task {
        CONST => print_consts(&classinfo),
        METHOD => {
            for (name, code_info) in code_blocks(&classinfo) {
                print_method(&classinfo, &name, &code_info);
            }
        },
        _ => ()
    };

}
