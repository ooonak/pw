#![allow(unused_imports)]

use chrono::{DateTime, Utc};
use prost::Message;
use pw::messages::network_interface::Rtpproto::Dhcp;
use std::time::Duration;

pub const BASE_KEY_EXPR: &str = "pw";
pub const GROUP_KEY_EXPR: &str = "1";
pub const MACHINE_KEY_EXPR: &str = "m";
pub const LIVELINESS_KEY_EXPR: &str = "l";
pub const COMMAND_KEY_EXPR: &str = "c";
pub const BOOTID_KEY_EXPR: &str = "b";

pub mod pw {
    pub mod messages {
        include!(concat!(env!("OUT_DIR"), "/pw.messages.rs"));
    }
}

pub fn serialize_machine(machine: &pw::messages::Machine) -> Vec<u8> {
    let mut buf = Vec::with_capacity(machine.encoded_len());

    // Unwrap is safe, we have reserved capacity in the vector.
    machine.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_machine(buf: &[u8]) -> Result<pw::messages::Machine, prost::DecodeError> {
    pw::messages::Machine::decode(buf)
}

pub fn stringify_duration(seconds: u64) -> String {
    let then = std::time::UNIX_EPOCH + Duration::from_secs(seconds);
    let datetime = DateTime::<Utc>::from(then);

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn stringify_message(machine: &pw::messages::Machine) -> Vec<(&str, String)> {
    vec![
        ("booted", stringify_duration(machine.boottime)),
        ("hostname", machine.hostname.clone()),
        ("kernel", machine.version.clone()),
        ("CPU", machine.cpu_model_name.clone()),
        (
            "RAM",
            format!("{} MB", (machine.physical_mem_total_kb / 1000)),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use std::{time::Duration, vec};

    use chrono::{DateTime, Utc};
    use pw::messages::NetworkInterface;

    use super::*;

    #[test]
    fn serialize_machine() {
        let machine = pw::messages::Machine {
            boottime: 12345678,
            ..Default::default()
        };

        let buffer = super::serialize_machine(&machine);

        let expected: Vec<u8> = vec![8, 206, 194, 241, 5];

        assert_eq!(buffer, expected);
    }

    #[test]
    fn deserialize_machine() {
        let buffer: Vec<u8> = vec![8, 206, 194, 241, 5];

        let machine = super::deserialize_machine(&buffer);

        let expected = pw::messages::Machine {
            boottime: 12345678,
            ..Default::default()
        };

        assert_eq!(machine.unwrap(), expected);
    }

    #[test]
    fn stringify_ok() {
        let input = pw::messages::Machine {
            boottime: 1731155405,
            bootid: "5be4b9be-6f40-4ba5-ab5d-72cf867cfa0d".to_string(),
            hostname: "raspberrypi".to_string(),
            version: "Linux version 6.6.51+rpt-rpi-v7 (serge@raspberrypi.com) (gcc-12 (Raspbian 12.2.0-14+rpi1) 12.2.0, GNU ld (GNU Binutils for Raspbian) 2.40) #1 SMP Raspbian 1:6.6.51-1+rpt3 (2024-10-08)".to_string(),
            cpu_model_name: "ARMv7 Processor rev 4 (v7l)".to_string(),
            physical_mem_total_kb: 943032,
            network_interface: Some( NetworkInterface { mac: 202481586980485, name: "eth0".to_string(), proto: Dhcp.into(), ipv4: 3232246377, subnet_mask: 24, broadcast: 3232246527, gateway: 3232246273 }),
        };

        let expected = vec![
            ( "booted", "2024-11-09 12:30:05".to_string() ),
            ( "hostname", "raspberrypi".to_string() ),
            ( "kernel", "Linux version 6.6.51+rpt-rpi-v7 (serge@raspberrypi.com) (gcc-12 (Raspbian 12.2.0-14+rpi1) 12.2.0, GNU ld (GNU Binutils for Raspbian) 2.40) #1 SMP Raspbian 1:6.6.51-1+rpt3 (2024-10-08)".to_string() ),
            ( "CPU", "ARMv7 Processor rev 4 (v7l)".to_string() ),
            ( "RAM", "943 MB".to_string() ),
        ];

        assert_eq!(stringify_message(&input), expected);
    }
}
