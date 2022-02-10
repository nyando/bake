extern crate binrw;

use std::io::Cursor;
use crate::structs::binrw::BinReaderExt;
use binrw::binrw;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;

#[binrw]
pub enum ConstPoolInfo {
    #[br(magic(7u8))] ConstClass {
        name_index: u16
    },
    #[br(magic(9u8))] ConstFieldRef {
        class_index: u16,
        name_and_type_index: u16
    },
    #[br(magic(10u8))] ConstMethodRef {
        class_index: u16,
        name_and_type_index: u16
    },
    #[br(magic(11u8))] ConstInterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16
    },
    #[br(magic(8u8))] ConstString {
        string_index: u16
    },
    #[br(magic(3u8))] ConstInt {
        bytes: u32
    },
    #[br(magic(4u8))] ConstFloat {
        bytes: u32
    },
    #[br(magic(5u8))] ConstLong {
        high_bytes: u32,
        low_bytes: u32
    },
    #[br(magic(6u8))] ConstDouble {
        high_bytes: u32,
        low_bytes: u32
    },
    #[br(magic(12u8))] ConstNameAndType {
        name_index: u16,
        descriptor_index: u16
    },
    #[br(magic(1u8))] ConstUTF8 {
        length: u16,
        #[br(count = length)]
        bytes: Vec<u8>
    },
    #[br(magic(15u8))] ConstMethodHandle {
        reference_kind: u8,
        reference_index: u16
    },
    #[br(magic(16u8))] ConstMethodType {
        descriptor_index: u16
    },
    #[br(magic(18u8))] ConstInvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16
    }
}

#[binrw]
pub struct FieldInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    #[br(count = attributes_count)]
    attributes: Vec<AttributeInfo>
}

#[binrw]
pub struct MethodInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    #[br(count = attributes_count)]
    attributes: Vec<AttributeInfo>
}

#[binrw]
pub struct AttributeInfo {
    attribute_name_index: u16,
    attribute_length: u32,
    #[br(count = attribute_length)]
    info: Vec<u8>
}

#[binrw]
pub struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16
}

#[binrw]
pub struct CodeAttribute {
    // attribute_name_index: u16,
    // attribute_length: u32,
    max_stack: u16,
    max_locals: u16,
    code_length: u32,
    #[br(count = code_length)]
    code: Vec<u8>,
    exception_table_length: u16,
    #[br(count = exception_table_length)]
    exception_table: Vec<ExceptionTableEntry>,
    attributes_count: u16,
    #[br(count = attributes_count)]
    attributes: Vec<AttributeInfo>
}

#[binrw]
pub struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constpool_count: u16,
    #[br(count = constpool_count - 1)]
    constpool: Vec<ConstPoolInfo>,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces_count: u16,
    #[br(count = interfaces_count)]
    interfaces: Vec<u16>,
    fields_count: u16,
    #[br(count = fields_count)]
    fields: Vec<FieldInfo>,
    methods_count: u16,
    #[br(count = methods_count)]
    methods: Vec<MethodInfo>,
    attributes_count: u16,
    #[br(count = attributes_count)]
    attributes: Vec<AttributeInfo>
}

pub fn read_classfile(path : &str) -> Result<ClassFile, std::io::Error> {
    let res = File::open(path);

    match res {
        Ok(mut file) => Ok(file.read_be().unwrap()),
        Err(err)     => Err(Error::from(err))
    }
}

pub fn utf8_constants(class : &ClassFile) -> HashMap<u16, &str> {
    let mut utf8_constpool : HashMap<u16, &str> = HashMap::new();

    for i in 0..class.constpool_count - 1 {

        let const_info : &ConstPoolInfo = &class.constpool[i as usize];

        match const_info {
            ConstPoolInfo::ConstUTF8 { length: _, bytes } => {
                let content : &str = std::str::from_utf8(bytes).unwrap();
                utf8_constpool.insert(i + 1, content);
            },
            _ => { }
        }

    }

    utf8_constpool
}

pub fn code_blocks(class: &ClassFile) -> HashMap<&str, Vec<u8>> {

    let mut codeblocks : HashMap<&str, Vec<u8>> = HashMap::new();

    let utf8_constpool = utf8_constants(class);

    for i in 0..class.methods_count {

        let method_info : &MethodInfo = &class.methods[i as usize];
        let method_name_index : u16 = method_info.name_index;
        let attr_info : &AttributeInfo = &method_info.attributes[0];
        let attr_name_index : u16 = attr_info.attribute_name_index;

        if utf8_constpool[&attr_name_index].eq("Code") {
            let method_name : &str = utf8_constpool[&method_name_index];
            let code_attr : CodeAttribute = Cursor::new(&attr_info.info).read_be().unwrap();
            codeblocks.insert(method_name, code_attr.code);
        }

    }

    codeblocks
}