extern crate bat;
extern crate clap;
extern crate hexyl;

use clap::{Subcommand, Parser};

mod structs;
use structs::*;

mod memory;
use memory::*;

mod opcodes;
use opcodes::*;

mod print;
use print::*;

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
                println!("{}", constoutput(&classinfo, &index, &value));
            }
        },
        Commands::Method { classfile } => {
            let classinfo = read_classfile(classfile)?;
            for (name, code_info) in codeblocks(&classinfo) {
                if name == INIT_SIG { continue; }
                let signature = methodstring(&name);
                print_method(&classinfo, &signature, &code_info);
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
                true,
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
