use std::collections::HashMap;

use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
/// Associates a bytecode value to its corresponding mnemonic and its number of bytecode arguments.
pub struct Op {
    #[serde(with = "hex")]
    pub opcode: [u8; 1],
    pub mnemonic: String,
    pub args: u8
}

///
/// Creates a list of JVM bytecode opcodes from a corresponding CSV file.
///
/// Returns a mapping of opcode to `Op` structure describing the operation.
///
pub fn op_map() -> HashMap<u8, Op> {
    let opcodes : String = include_str!("opcodes.csv").to_string();
    let mut rdr = ReaderBuilder::new().from_reader(opcodes.as_bytes());
    let mut opmap = HashMap::new();

    for result in rdr.deserialize() {
        let op : Op = result.unwrap();
        opmap.insert(op.opcode[0], op);
    }

    opmap
}
