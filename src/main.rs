extern crate clap;
extern crate hexyl;

use clap::{Subcommand, Parser};

mod structs;
use structs::*;

mod opcodes;
use opcodes::*;

mod memory;
use memory::*;

mod uart;
use uart::*;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract constants from JVM class file
    Consts {
        /// Path of the class file to parse
        #[clap(short, long)]
        classfile: String
    },
    /// Extract method information from JVM class file
    Method {
        /// Path of the class file to parse
        #[clap(short, long)]
        classfile: String
    },
    /// Generate Bali binary from JVM class file
    Binary {
        /// Path of the class file to convert to binary 
        #[clap(short, long)]
        classfile: String
    },
    /// Write Bali binary to serial Bali device
    Serial {
        /// Path of the binary file to transfer to Bali device
        #[clap(short, long)]
        bin: String,
        /// Serial device identifier of target Bali device
        #[clap(short, long)]
        device: String
    }
}

fn print_const(classinfo : &ClassFile, index : &u16, value : &ConstPoolValue) {
    match value {
        ConstPoolValue::Class(name_ref) => println!("{}: {}", index, name_ref),
        ConstPoolValue::Integer(int_const) => println!("{}: {}", index, int_const),
        ConstPoolValue::MethodRef(_, desc_ref) => println!("{}: {}", index, parse_method_signature(&classinfo, desc_ref).unwrap()),
        ConstPoolValue::NameAndType(name_ref, type_ref) => println!("{}: {}, {}", index, name_ref, type_ref),
        ConstPoolValue::UTF8String(str_const) => println!("{}: {}", index, str_const)
    };
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
                    println!("{}: {}", op.mnemonic, methodrefs.get_by_left(&arg).unwrap());
                } else {
                    println!("{}: {:#06x}", op.mnemonic, arg);
                }
            },
            _ => println!("unknown opcode")
        };

    }

}

fn read_binary(path : &str) -> Vec<u8> {
    let res = File::open(path);
    let mut buffer = Vec::new();

    res.unwrap().read_to_end(&mut buffer).unwrap();

    buffer
}

fn main() -> std::io::Result<()> {

    let task = Args::parse();

    match &task.command {
        Commands::Consts { classfile } => {
            let classinfo = read_classfile(classfile)?;
            for (index, value) in constants(&classinfo) {
                print_const(&classinfo, &index, &value);
            }
        },
        Commands::Method { classfile } => {
            let classinfo = read_classfile(classfile)?;
            for (name, code_info) in codeblocks(&classinfo) {
                print_method(&classinfo, &name, &code_info);
            }
        },
        Commands::Binary { classfile } => {
            let classinfo = read_classfile(classfile)?;
            let binary = binarygen(&classinfo);
            let outpath = Path::new(&classfile).with_extension("bali.out");
            let mut buffer = File::create(outpath.to_str().unwrap())?;

            buffer.write_all(&binary)?;

            hexyl::Printer::new(
                &mut std::io::stdout(),
                false,
                hexyl::BorderStyle::Ascii,
                true
            ).print_all(std::io::Cursor::new(binary)).unwrap();
        },
        Commands::Serial { bin, device } => {
            let binary = read_binary(bin);
            let mut port = open_serial(device);
            binwrite(&mut port, &binary)?;
        }
    };

    Ok(())
}
