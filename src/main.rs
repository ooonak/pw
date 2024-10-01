use std::fs;
use serde::{Deserialize, Serialize};
use rmp_serde::{Deserializer, Serializer};

use mac_address::{get_mac_address, MacAddress};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Machine {
    mac: [u8; 6],
    boot: String,
}   

fn identity() -> Machine {
    let mut mac = [0, 0, 0, 0, 0, 0];

    match get_mac_address() {
        Ok(Some(ma)) => { mac = ma.bytes(); }
        _ => {}
    }

    let mut contents = fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap_or_default();
    if contents.ends_with('\n') {
        contents.pop();
    }

    Machine { mac: mac, boot: contents }
}

fn main() {
    let machine = identity();
    
    let mut buf = Vec::new();
    machine.serialize(&mut Serializer::new(&mut buf)).unwrap();

    println!("{:?}", machine);
    println!("MessagePack ({} bytes) {:x?}", buf.len(), buf);
}
