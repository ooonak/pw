use std::fs;
use serde::{Deserialize, Serialize};
use rmp_serde::{Deserializer, Serializer};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Machine {
    boot_id: String,
}   

fn boot_id() -> String {
    let mut contents = fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap_or_default();
    if contents.ends_with('\n') {
        contents.pop();
    }

    contents
}

fn main() {
    let machine = Machine { boot_id: boot_id().into() };
    
    let mut buf = Vec::new();
    machine.serialize(&mut Serializer::new(&mut buf)).unwrap();

    println!("{:?}", machine);
    println!("MessagePack ({} bytes) {:x?}", buf.len(), buf);
}
