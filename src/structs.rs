extern crate binrw;

use std::io::Cursor;
use crate::structs::binrw::BinReaderExt;
use binrw::binrw;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;

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

pub enum ConstPoolValue {
    UTF8String(String),
    Integer(i32)
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
    pub signature: String,
    pub max_stack: u16,
    pub max_locals: u16,
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
        Err(err)     => Err(Error::from(err))
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
            }
            _ => { }
        }

    }

    constpool
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
pub fn code_blocks(class: &ClassFile) -> HashMap<String, BaliCode> {

    let mut codeblocks : HashMap<String, BaliCode> = HashMap::new();

    let utf8_constpool : HashMap<u16, String> = constants(class)
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
            let method_name : String = utf8_constpool[&method_name_index].to_string();
            let method_desc : String = utf8_constpool[&method_desc_index].to_string();
            let code_attr : CodeAttribute = Cursor::new(&attr_info.info).read_be().unwrap();
            let code_info = BaliCode {
                signature: method_desc.to_string(),
                max_stack: code_attr.max_stack,
                max_locals: code_attr.max_locals,
                code: code_attr.code
            };
            // TODO: method name is not a unique identifier of a method, needs descriptor (overloading)
            codeblocks.insert(method_name, code_info);
        }

    }

    codeblocks
}