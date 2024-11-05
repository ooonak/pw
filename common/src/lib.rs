use prost::Message;

pub mod pw {
    pub mod messages {
        include!(concat!(env!("OUT_DIR"), "/pw.messages.rs"));
    }
}

pub fn serialize_machine(machine: &pw::messages::Machine) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(machine.encoded_len());
    // Unwrap is safe, we have reserved capacity in the vector.
    machine.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_machine(buf: &[u8]) -> Result<pw::messages::Machine, prost::DecodeError> {
    pw::messages::Machine::decode(buf)
}
