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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_machine() {
        let mut machine = pw::messages::Machine::default();
        machine.mac = 12345678;
        
        let buffer = super::serialize_machine(&machine);

        let expected: Vec<u8> = vec![8, 206, 194, 241, 5];

        assert_eq!(buffer, expected);
    }

    #[test]
    fn deserialize_machine() {
        let buffer: Vec<u8> = vec![8, 206, 194, 241, 5];

        let machine = super::deserialize_machine(&buffer);
        
        let mut expected = pw::messages::Machine::default();
        expected.mac = 12345678;

        assert_eq!(machine.unwrap(), expected);
    }
}
