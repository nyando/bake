use crate::structs::ClassFile;
use crate::structs::{ConstPoolValue, constants, codeblocks, methodrefs};
use crate::opcodes::opmap;

use bimap::BiBTreeMap;

use std::collections::HashMap;
use std::collections::btree_map::BTreeMap;

pub const INIT_SIG : &str = "<init>()V";
pub const MAIN_SIG : &str = "main([Ljava/lang/String;)V";

const LUTENTRY : usize = 4;

fn memlayout(classinfo : &ClassFile) -> (BiBTreeMap<u16, String>, u16) {
    let codeblocks = codeblocks(classinfo);

    let maincode = &codeblocks[MAIN_SIG];
    let mainsize : u16 = maincode.code.len() as u16;

    let mut currentaddr : u16 = 0;
    let mut methodaddrs : BiBTreeMap<u16, String> = BiBTreeMap::new();

    methodaddrs.insert(currentaddr, MAIN_SIG.to_string());
    currentaddr += mainsize;

    for (name, codeblock) in codeblocks {
        
        // main is already in the map, <init> ignored
        if name == MAIN_SIG || name == INIT_SIG { continue; }

        let codesize : u16 = codeblock.code.len() as u16;
        methodaddrs.insert(currentaddr, name);
        currentaddr += codesize;

    }

    (methodaddrs, currentaddr)
}

fn luts(classinfo : &ClassFile) -> (Vec<u8>, BTreeMap<u16, u16>, BTreeMap<String, u16>) {
    let codeblocks = codeblocks(classinfo);
    let (memlayout, _) = memlayout(classinfo);

    let ints : BTreeMap<u16, i32> = constants(classinfo)
        .into_iter()
        .filter(|(_, v)| matches!(v, ConstPoolValue::Integer(_)))
        .map(|(k, v)| if let ConstPoolValue::Integer(value) = v { (k, value) } else { (0, 0) })
        .collect();

    let method_entry_count : usize = memlayout.len();
    let consts_entry_count : usize = ints.len();
    let lutsize : usize = LUTENTRY * method_entry_count + LUTENTRY * consts_entry_count;

    let mut methodlut : Vec<u8> = Vec::with_capacity(lutsize);
    let mut nameindex : BTreeMap<String, u16> = BTreeMap::new();
    for (i, (methodaddr, methodname)) in memlayout.into_iter().enumerate() {
        methodlut.push(((lutsize as u16 + methodaddr as u16) >> 8) as u8);
        methodlut.push(((lutsize as u16 + methodaddr as u16) & 0xff) as u8);
        methodlut.push(if methodname == MAIN_SIG { 0x00_u8 } else { codeblocks[&methodname].argcount as u8 });
        methodlut.push(codeblocks[&methodname].max_locals.try_into().unwrap());
        nameindex.insert(methodname, i.try_into().unwrap());
    }

    let mut constmap : BTreeMap<u16, u16> = BTreeMap::new();
    let mut memindex = method_entry_count as u8;
    for (poolindex, intvalue) in &ints {
        methodlut.push((intvalue >> 24) as u8);
        methodlut.push((intvalue >> 16) as u8);
        methodlut.push((intvalue >> 8) as u8);
        methodlut.push(*intvalue as u8);

        constmap.insert(*poolindex, memindex.into());
        memindex += 1;
    }

    (methodlut, constmap, nameindex)
}

///
/// Generate binary stream to write to Bali processor program memory.
///
/// Returns byte vector for writing to output file.
///
pub fn binarygen(classinfo : &ClassFile) -> Vec<u8> {
    let methodrefs = methodrefs(classinfo);
    let (memlayout, codesize) = memlayout(classinfo);
    let (mut methodlut, intrefs, methodaddrs) = luts(classinfo);
    let opmap = opmap();

    // map method reference index to Bali program memory address
    let mut refaddr : HashMap<u16, u16> = HashMap::new();
    for (methodref, methodname) in methodrefs {
        if methodname == INIT_SIG { continue; }
        refaddr.insert(methodref, methodaddrs[&methodname]);
    }

    // replace invokestatic address arguments with Bali memory addresses
    let codeblocks = codeblocks(classinfo);
    let memsize = methodlut.len() as u16 + codesize;
    let mut mem = Vec::with_capacity(memsize.into());
    mem.append(&mut methodlut);

    for (_, methodname) in memlayout {
        if methodname == INIT_SIG { continue; }

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
                let oldref = ((code_old[i + 1] as u16) << 8) | code_old[i + 2] as u16;
                let newref = refaddr[&oldref] as u16;
                code_new[i + 1] = (newref >> 8) as u8;
                code_new[i + 2] = (newref & 0xff) as u8;
            }

            if op.mnemonic == "ldc" {
                let oldref = code_old[i + 1] as u16;
                let newref = intrefs[&oldref];
                code_new[i + 1] = newref as u8;
            }

            if methodname == MAIN_SIG && op.mnemonic == "return" {
                code_new[i] = 0xFF_u8; // NOP
            }

        }
       
        mem.append(&mut code_new);
    }

    mem
}
