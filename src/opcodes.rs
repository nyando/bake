use std::collections::HashMap;

use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Op {
    #[serde(with = "hex")]
    pub opcode: [u8; 1],
    pub mnemonic: String,
    pub args: u8
}

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
