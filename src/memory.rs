extern crate bimap;

use crate::ClassFile;
use crate::{codeblocks, opmap, methodrefs};

use bimap::BiBTreeMap;

use std::collections::HashMap;

const INIT_SIG : &str = "<init>()V";
const MAIN_SIG : &str = "main([Ljava/lang/String;)V";

///
/// Creates memory layout for Bali program memory out of a JVM class file.
///
/// Returns:
///
///   1. A bidirectional map that maps a method's address to its signature.
///   2. The total size of the memory block for the given class file.
///
pub fn memlayout(classinfo : &ClassFile) -> (BiBTreeMap<u16, String>, u16) {
    let codeblocks = codeblocks(&classinfo);

    let maincode = &codeblocks[MAIN_SIG];
    let mainsize : u16 = maincode.code.iter().count().try_into().unwrap();

    let mut currentaddr : u16 = 0;
    let mut methodaddrs : BiBTreeMap<u16, String> = BiBTreeMap::new();

    methodaddrs.insert(currentaddr, MAIN_SIG.to_string());
    currentaddr += mainsize;

    for (name, codeblock) in codeblocks {
        
        // main is already in the map, <init> ignored
        if name == MAIN_SIG || name == INIT_SIG { continue; }

        let codesize : u16 = codeblock.code.iter().count().try_into().unwrap();
        methodaddrs.insert(currentaddr, name);
        currentaddr += codesize;

    }

    (methodaddrs, currentaddr)
}

///
/// Generate binary stream to write to Bali processor program memory.
///
/// Returns byte vector for writing to output file.
///
pub fn binarygen(classinfo : &ClassFile) -> Vec<u8> {
    let methodrefs = methodrefs(&classinfo);
    let memlayout = memlayout(&classinfo);
    let opmap = opmap();

    // map method reference index to Bali program memory address
    let mut refaddr : HashMap<u16, u16> = HashMap::new();
    for (methodref, methodname) in methodrefs {
        if methodname == INIT_SIG { continue; }
        refaddr.insert(methodref, *memlayout.0.get_by_right(&methodname).unwrap());
    }

    // replace invokestatic address arguments with Bali memory addresses
    let codeblocks = codeblocks(&classinfo);
    let mut mem = Vec::with_capacity(memlayout.1.into());
    for (_, methodname) in memlayout.0 {
        let code_old = codeblocks[&methodname].code.to_vec();
        let mut code_new = codeblocks[&methodname].code.to_vec();

        let mut argcount = 0;
        for (i, opcode) in code_old.iter().enumerate() {
            // skip arguments in parsing opcodes
            if argcount > 0 {
                argcount -= 1;
                continue;
            }
            
            let op = &opmap[opcode];
            argcount = op.args;

            if op.mnemonic == "invokestatic" {
                let arg = (code_old[i + 1] as u16) << 8 | (code_old[i + 2] as u16);
                let newref = refaddr[&arg];

                code_new[i + 1] = (newref >> 8) as u8;
                code_new[i + 2] = (newref & 0xff) as u8;
            }
        }
        mem.append(&mut code_new);
    }

    mem
}
