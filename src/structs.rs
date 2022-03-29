extern crate binrw;
extern crate bimap;

use bimap::BiMap;

use binrw::{binrw, BinReaderExt};

use std::collections::BTreeMap;

use std::io::{Cursor, Error, ErrorKind};
use std::fs::File;

#[binrw]
/// Structures containing constants, metadata of the JVM class file.
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

/// Structures for constant pool objects needed for Bali binaries.
pub enum ConstPoolValue {
    Class(u16),
    Integer(i32),
    MethodRef(u16, u16),
    NameAndType(u16, u16),
    UTF8String(String)
}

#[binrw]
/// Structure containing information about the class fields.
pub struct FieldInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    #[br(count = attributes_count)]
    attributes: Vec<AttributeInfo>
}

#[binrw]
/// Structure containing information about the class methods.
pub struct MethodInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    #[br(count = attributes_count)]
    attributes: Vec<AttributeInfo>
}

#[binrw]
/// Generic container structure for an attribute of a class, method, field, etc.
pub struct AttributeInfo {
    attribute_name_index: u16,
    attribute_length: u32,
    #[br(count = attribute_length)]
    info: Vec<u8>
}

#[binrw]
/// Exception table entry structure that forms part of a code attribute.
pub struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16
}

#[binrw]
/// JVM class file code attribute structure.
pub struct CodeAttribute {
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

/// Structure containing the relevant method information for the Bali processor.
pub struct BaliCode {
    pub max_stack: u16,
    pub max_locals: u16,
    pub argcount: u16,
    pub code: Vec<u8>
}

#[binrw]
/// Top-level structure for a JVM class file.
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


///
/// Creates a `ClassFile` from the JVM class file given in the path.
///
/// Returns an error result if the file could not be parsed, otherwise returns a `ClassFile` structure.
///
pub fn read_classfile(path : &str) -> Result<ClassFile, Error> {
    let res = File::open(path);

    match res {
        Ok(mut file) => Ok(file.read_be().unwrap()),
        Err(err)     => Err(err)
    }
}

///
/// Extracts UTF-8 String and integer constants from class file constant pool definition.
///
/// Returns a mapping of the constant pool index (1-based) to the corresponding UTF-8 string as a string slice or i32 integer.
///
pub fn constants(class : &ClassFile) -> BTreeMap<u16, ConstPoolValue> {
    let mut constpool : BTreeMap<u16, ConstPoolValue> = BTreeMap::new();

    for i in 0..class.constpool_count - 1 {

        let const_info : &ConstPoolInfo = &class.constpool[i as usize];

        match const_info {
            ConstPoolInfo::ConstUTF8 { length: _, bytes } => {
                let content : &str = std::str::from_utf8(bytes).unwrap();
                constpool.insert(i + 1, ConstPoolValue::UTF8String(content.to_string()));
            },
            ConstPoolInfo::ConstInt { bytes } => {
                constpool.insert(i + 1, ConstPoolValue::Integer(*bytes as i32));
            },
            ConstPoolInfo::ConstMethodRef { class_index, name_and_type_index } => {
                constpool.insert(i + 1, ConstPoolValue::MethodRef(*class_index, *name_and_type_index));
            },
            ConstPoolInfo::ConstNameAndType { name_index, descriptor_index } => {
                constpool.insert(i + 1, ConstPoolValue::NameAndType(*name_index, *descriptor_index));
            }
            ConstPoolInfo::ConstClass { name_index } => {
                constpool.insert(i + 1, ConstPoolValue::Class(*name_index));
            }
            _ => { }
        }

    }

    constpool
}

///
/// Parse method signature given the `NameAndType` constant reference of its method reference.
///
/// Returns a Result of the string containing the method signature, or an error if the corresponding reference isn't found.
///
pub fn parse_method_signature(classinfo : &ClassFile, desc_ref : &u16) -> Result<String, Error> {
    let constpool = constants(classinfo);
    let mut signature = "".to_string();
    let error = Error::new(ErrorKind::Other, "could not parse method reference");

    if let ConstPoolValue::NameAndType(name_ref, type_ref) = constpool[desc_ref] {
        if let ConstPoolValue::UTF8String(method_name) = &constpool[&name_ref] {
            signature.push_str(method_name);
        } else { return Err(error); }
        if let ConstPoolValue::UTF8String(type_signature) = &constpool[&type_ref] {
            signature.push_str(type_signature);
        } else { return Err(error); }
    }

    Ok(signature)
}

///
/// Create mapping of method reference indices to corresponding method signature strings.
///
/// Returns a map of integers (constpool indices) to strings (method signatures).
///
pub fn methodrefs(classinfo : &ClassFile) -> BiMap<u16, String> {
    let constpool = constants(classinfo);
    let mut refmap : BiMap<u16, String> = BiMap::new();

    for (index, value) in &constpool {
        if let ConstPoolValue::MethodRef(_, desc_ref) = value {
            refmap.insert(*index, parse_method_signature(classinfo, desc_ref).unwrap());
        }
    }

    refmap
}

///
/// Extracts method names, information, and bytecode from class file structure.
///
/// Returns a mapping of the method names in the class to a `BaliCode` structure containing:
///
/// - maximum stack depth of the method
/// - maximum number of local variables used by the method
/// - vector of method bytecode
///
pub fn codeblocks(class: &ClassFile) -> BTreeMap<String, BaliCode> {

    let mut codeblocks : BTreeMap<String, BaliCode> = BTreeMap::new();

    let utf8_constpool : BTreeMap<u16, String> = constants(class)
            .into_iter()
            .filter(|(_, v)| matches!(v, ConstPoolValue::UTF8String(_)))
            .map(|(k, v)| if let ConstPoolValue::UTF8String(value) = v { (k, value) } else { (k, "".to_string()) })
            .collect();

    for i in 0..class.methods_count {

        let method_info : &MethodInfo = &class.methods[i as usize];
        let method_name_index : u16 = method_info.name_index;
        let method_desc_index : u16 = method_info.descriptor_index;
        let attr_info : &AttributeInfo = &method_info.attributes[0];
        let attr_name_index : u16 = attr_info.attribute_name_index;

        if utf8_constpool[&attr_name_index].eq("Code") {
            let mut method_name : String = utf8_constpool[&method_name_index].to_string();
            let method_desc : String = utf8_constpool[&method_desc_index].to_string();
            let code_attr : CodeAttribute = Cursor::new(&attr_info.info).read_be().unwrap();
            let code_info = BaliCode {
                max_stack: code_attr.max_stack,
                max_locals: code_attr.max_locals,
                argcount: parse_argcount(&method_desc),
                code: replace_iinc(&code_attr.code)
            };
            method_name.push_str(&method_desc);
            codeblocks.insert(method_name, code_info);
        }

    }

    codeblocks
}

const IINC   : u8 = 0x84;
const ILOAD  : u8 = 0x15;
const BIPUSH : u8 = 0x10;
const IADD   : u8 = 0x60;
const ISTORE : u8 = 0x36;

fn replace_iinc(code: &Vec<u8>) -> Vec<u8> {
    let mut code_out : Vec<u8> = Vec::new();

    let mut argc = 0;
    for i in 0..code.len() {
        if argc > 0 {
            argc = argc - 1;
            continue;
        }

        if code[i] == IINC {
            let index = code[i + 1];
            let byte = code[i + 2];
            let mut iinc_replace = vec!(ILOAD, index, BIPUSH, byte, IADD, ISTORE, index);
            code_out.append(&mut iinc_replace);
            argc = 2;
        } else {
            code_out.push(code[i]);
        }
    }

    code_out
}

fn parse_argcount(method_sig: &str) -> u16 {

    let split : Vec<&str> = method_sig.split(|c| (c == '(') || (c == ')')).collect();

    let mut argcount = 0;
    let arglist = &split[1];
    for c in arglist.chars() {
        // count the number of arguments == letters between parentheses of a method signature
        if c == '[' { continue; } else { argcount = argcount + 1; }
    }

    argcount
}