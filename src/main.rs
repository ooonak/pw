mod data_types;

use crate::data_types::Machine;
use rmp_serde::Serializer;
use serde::Serialize;

fn main() {
    let mut machine = Machine::new();
    machine.load();

    let mut buf = Vec::new();
    machine.serialize(&mut Serializer::new(&mut buf)).unwrap();

    println!("{:?}", machine);
    println!("MessagePack ({} bytes)", buf.len());
}
