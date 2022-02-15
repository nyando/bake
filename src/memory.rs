extern crate bimap;

use crate::ClassFile;
use crate::{codeblocks, opmap, methodrefs};

use bimap::BiBTreeMap;

use std::collections::HashMap;

const INIT_SIG : &str = "<init>()V";
const MAIN_SIG : &str = "main([Ljava/lang/String;)V";
const LUTENTRY : u8   = 3;

fn memlayout(classinfo : &ClassFile) -> (BiBTreeMap<u16, String>, u16) {
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

fn methodlut(classinfo : &ClassFile) -> Vec<u8> {
    let codeblocks = codeblocks(&classinfo);
    let (memlayout, _) = memlayout(&classinfo);
    let lutsize = LUTENTRY * memlayout.len() as u8;

    let mut methodlut : Vec<u8> = Vec::with_capacity(lutsize.into());

    for (methodaddr, methodname) in memlayout {
        methodlut.push(((lutsize as u16 + methodaddr as u16) >> 8) as u8);
        methodlut.push(((lutsize as u16 + methodaddr as u16) & 0xff) as u8);
        methodlut.push(codeblocks[&methodname].max_locals.try_into().unwrap());
    }

    methodlut
}

///
/// Generate binary stream to write to Bali processor program memory.
///
/// Returns byte vector for writing to output file.
///
pub fn binarygen(classinfo : &ClassFile) -> Vec<u8> {
    let methodrefs = methodrefs(&classinfo);
    let (memlayout, codesize) = memlayout(&classinfo);
    let mut methodlut = methodlut(&classinfo);
    let opmap = opmap();

    // map method reference index to Bali program memory address
    let mut refaddr : HashMap<u16, u16> = HashMap::new();
    for (methodref, methodname) in methodrefs {
        if methodname == INIT_SIG { continue; }
        refaddr.insert(methodref, *memlayout.get_by_right(&methodname).unwrap());
    }

    // replace invokestatic address arguments with Bali memory addresses
    let codeblocks = codeblocks(&classinfo);
    let memsize = methodlut.len() as u16 + codesize;
    let mut mem = Vec::with_capacity(memsize.into());
    mem.append(&mut methodlut);

    for (pos, (_, methodname)) in memlayout.iter().enumerate() {
        let code_old = codeblocks[methodname].code.to_vec();
        let mut code_new = codeblocks[methodname].code.to_vec();

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
                let newref = pos as u16;
                code_new[i + 1] = (newref >> 8) as u8;
                code_new[i + 2] = (newref & 0xff) as u8;
            }
        }
        mem.append(&mut code_new);
    }

    mem
}
