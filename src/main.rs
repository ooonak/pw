use std::fs;
use serde::{Deserialize, Serialize};
use rmp_serde::Serializer;
        
#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Machine {
    boot: String,
}   

fn identity() -> Machine {
    let mut contents = fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap_or_default();
    if contents.ends_with('\n') {
        contents.pop();
    }

    Machine { boot: contents }
}

fn main() {
    let machine = identity();
    
    let mut buf = Vec::new();
    machine.serialize(&mut Serializer::new(&mut buf)).unwrap();

    println!("{:?}", machine);
    println!("MessagePack ({} bytes) {:x?}", buf.len(), buf);
}
