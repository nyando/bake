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
        classfile: String,
        /// Set this flag to print the hex output of the binary
        #[clap(short, long)]
        output: bool
    },
    /// Generate Bali file for use with SystemVerilog testbenches from JVM class file
    Testfile {
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
        device: String,
        /// Set this flag if Bali device expects 16 bit program length
        #[clap(short, long)]
        long: bool
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
        Commands::Binary { classfile, output } => {
            let classinfo = read_classfile(classfile)?;
            let binary = binarygen(&classinfo);
            let outpath = Path::new(&classfile).with_extension("bali.out");
            let mut buffer = File::create(outpath.to_str().unwrap())?;

            buffer.write_all(&binary)?;

            if *output {
                hexyl::Printer::new(
                    &mut std::io::stdout(),
                    true,
                    hexyl::BorderStyle::Ascii,
                    true
                ).print_all(std::io::Cursor::new(binary)).unwrap();
            }
        },
        Commands::Testfile { classfile } => {
            let classinfo = read_classfile(classfile)?;
            let binary = binarygen(&classinfo);
            let outpath = Path::new(&classfile).with_extension("mem");
            let mut buffer = File::create(outpath.to_str().unwrap())?;

            let mut output = String::new();
            for byte in binary {
                output.push_str(&format!("{:02x?} ", byte));
            }

            write!(buffer, "{}", output)?;
        }
        Commands::Serial { bin, device, long } => {
            let binary = read_binary(bin);
            let mut port = open_serial(device);
            
            if let Ok(turnaround) = binwrite(&mut port, &binary, *long) {
                println!("{}", turnaround);
            }
        }
    };

    Ok(())
}
