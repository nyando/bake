use crate::MAIN_SIG;
use crate::{methodrefs, opmap, parse_method_signature};
use crate::{BaliCode, ConstPoolValue, ClassFile, Op};

pub fn constoutput(classinfo : &ClassFile, index : &u16, value : &ConstPoolValue) -> String {
    match value {
        ConstPoolValue::Class(name_ref)                 => format!("{}: {}", index, name_ref),
        ConstPoolValue::Integer(int_const)              => format!("{}: {}", index, int_const),
        ConstPoolValue::MethodRef(_, desc_ref)          => format!("{}: {}", index, parse_method_signature(classinfo, desc_ref).unwrap()),
        ConstPoolValue::NameAndType(name_ref, type_ref) => format!("{}: {}, {}", index, name_ref, type_ref),
        ConstPoolValue::UTF8String(str_const)           => format!("{}: {}", index, str_const)
    }
}

fn fieldtype(typeid : &str) -> String {
    return match typeid {
        "B" => "byte".to_string(),
        "C" => "char".to_string(),
        "I" => "int".to_string(),
        "S" => "short".to_string(),
        "V" => "void".to_string(),
        "Z" => "boolean".to_string(),
        "[B" => "byte[]".to_string(),
        "[C" => "char[]".to_string(),
        "[I" => "int[]".to_string(),
        "[S" => "short[]".to_string(),
        "[Z" => "boolean[]".to_string(),
        &_ => panic!("Invalid field type identifier pattern")
    }
}

pub fn methodstring(sig : &str) -> String {

    if sig == MAIN_SIG { return "void main(String[])".to_string(); }

    let signature = sig.to_string();
    let split : Vec<String> = signature.split(|c| (c == '(') || (c == ')'))
        .map(|s| s.to_string())
        .collect();

    let name = &split[0];
    let returntype = fieldtype(&split[2]);

    let mut arglist : Vec<String> = Vec::new();
    let mut is_array = false;
    for c in split[1].chars() {
        match c {
            '[' => {
                is_array = true;
            },
            _ => {
                let expr = if is_array {
                    format!("[{}", c)
                } else {
                    c.to_string()
                };
                arglist.push(fieldtype(&expr));
                is_array = false;
            }
        }
    }

    let mut argstring = String::new();
    arglist.iter().fold(true, |first, elem| {
        if !first { argstring.push_str(", "); }
        argstring.push_str(elem);
        false
    });

    format!("{} {}({})", returntype, name, argstring)
}

pub fn print_method(classinfo : &ClassFile, name : &str, code_info : &BaliCode) {

    let opmap = opmap();
    let mut code_iter = code_info.code.iter();
    let methodrefs = methodrefs(classinfo);

    let mut output : Vec<u8> = Vec::new();

    output.append(&mut format!("stack size:                {:>3}\n", code_info.max_stack).into_bytes());
    output.append(&mut format!("local variable array size: {:>3}\n\n", code_info.max_locals).into_bytes());

    let mut addr = 0;
    while let Some(opcode) = code_iter.next() {
        
        let op : &Op = &opmap[opcode];
        
        match op.args {
            0 => output.append(&mut format!("{:>3x}: {:15}\n", addr, op.mnemonic).into_bytes()),
            1 => {
                let arg = code_iter.next().unwrap();
                output.append(&mut format!("{:>3x}: {:15} {:#04x}\n", addr, op.mnemonic, arg).into_bytes());
            },
            2 => {
                let arg1 = code_iter.next().unwrap();
                let arg2 = code_iter.next().unwrap();
                let arg : u16 = (*arg1 as u16) << 8 | (*arg2 as u16);

                // if static method invocation, print signature of invoked method
                if op.mnemonic == "invokestatic" {
                    output.append(&mut format!("{:>3x}: {:15} {}\n", addr, op.mnemonic, methodrefs.get_by_left(&arg).unwrap()).into_bytes());
                } else {
                    output.append(&mut format!("{:>3x}: {:15} {:#06x}\n", addr, op.mnemonic, arg).into_bytes());
                }
            },
            _ => panic!("unknown opcode")
        };

        addr += op.args + 1;
    }
    
    let inputstruct = bat::Input::from_reader(std::io::Cursor::new(&output))
        .title(name.to_string());

    bat::PrettyPrinter::new()
        .input(inputstruct)
        .header(true)
        .grid(true)
        .line_numbers(true)
        .print()
        .unwrap();
}