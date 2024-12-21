use common::pw;

use super::{error::MachineError, utils::{
    get_default_route_info, get_ip_address_info, ip_from_string, mac_from_string, parse_lines,
    parse_lines_no_separator, parse_number, parse_number_no_separator, read_lines,
}, Machine};

/// Struct that encapsulates data.
pub struct LinuxMachine {
    machine_info: common::pw::messages::Machine,
}

/// Concrete implementation of machine trait, a Linux machine.
impl LinuxMachine {
    pub fn new() -> Result<Self, MachineError> {
        let machine_info = load();
        if machine_info.network_interface.is_none() || machine_info.network_interface.as_ref().unwrap().mac == 0 {
            return Err(MachineError { message: "Could not load valid MAC".to_owned(), line: line!(), column: column!() });
        }

        Ok(Self { machine_info })
    }
}

impl Machine for LinuxMachine {
    fn bootid(&self) -> &str {
        &self.machine_info.bootid
    }

    fn mac(&self) -> u64 {
        // Safe to unwrap, new has checked for existence of network_interface.
        self.machine_info.network_interface.as_ref().unwrap().mac
    }
}

fn load() -> common::pw::messages::Machine {
    let mut machine = common::pw::messages::Machine::default();

    if let Some(value) = parse_boottime() {
        machine.boottime = value;
    }

    if let Some(value) = parse_boot_id() {
        machine.bootid = value;
    }

    if let Some(value) = parse_hostname() {
        machine.hostname = value;
    }

    if let Some(value) = parse_version() {
        machine.version = value;
    }

    if let Some(value) = parse_cpuinfo() {
        machine.cpu_model_name = value;
    }

    if let Some(value) = parse_mem_size() {
        machine.physical_mem_total_kb = value;
    }

    if let Some(value) = parse_network_info() {
        machine.network_interface = Some(value);
    }

    machine
}

fn parse_boottime() -> Option<u64> {
    if let Ok(all_lines) = read_lines("/proc/stat") {
        let elements = vec![("btime", false)];
        let lines = parse_lines_no_separator(all_lines, elements);

        return parse_number_no_separator(&lines[0]);
    }

    None
}

fn parse_boot_id() -> Option<String> {
    if let Ok(lines) = read_lines("/proc/sys/kernel/random/boot_id") {
        return parse_lines(lines, vec![], true).pop();
    }

    None
}

fn parse_hostname() -> Option<String> {
    if let Ok(lines) = read_lines("/proc/sys/kernel/hostname") {
        return parse_lines(lines, vec![], true).pop();
    }

    None
}

fn parse_version() -> Option<String> {
    if let Ok(lines) = read_lines("/proc/version") {
        return parse_lines(lines, vec![], true).pop();
    }

    None
}

fn parse_cpuinfo() -> Option<String> {
    if let Ok(lines) = read_lines("/proc/cpuinfo") {
        let elements = vec![("model name", false)];
        let lines = parse_lines(lines, elements, true);

        return Some(lines[0].clone());
    }

    None
}

fn parse_mem_size() -> Option<u32> {
    if let Ok(all_lines) = read_lines("/proc/meminfo") {
        let elements = vec![("MemTotal:", false)];
        let lines = parse_lines(all_lines, elements, true);

        return parse_number(&lines[0]).ok();
    }

    None
}

fn parse_network_info() -> Option<pw::messages::NetworkInterface> {
    if let Some(route_info) = get_default_route_info() {
        if let Some(address_info) = get_ip_address_info(&route_info[0]) {
            let mut info = common::pw::messages::NetworkInterface {
                name: route_info[0].to_string(),
                ..Default::default()
            };

            if route_info[1] == "static" {
                info.set_proto(pw::messages::network_interface::Rtpproto::Static);
            } else if route_info[1] == "dhcp" {
                info.set_proto(pw::messages::network_interface::Rtpproto::Dhcp);
            } else {
                info.set_proto(pw::messages::network_interface::Rtpproto::Unknown);
            }

            info.mac = mac_from_string(&address_info[3]);
            info.ipv4 = ip_from_string(&address_info[0]);
            if let Ok(mask) = parse_number(&address_info[1]) {
                info.subnet_mask = mask;
            }
            info.broadcast = ip_from_string(&address_info[2]);
            info.gateway = ip_from_string(&route_info[2]);

            return Some(info);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::platform::utils::{parse_lines_no_separator, parse_number_no_separator};

    use super::*;

    #[test]
    fn boottime() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/stat");

        let all_lines = read_lines(path).expect("Could not read");
        let elements = vec![("btime", false)];
        let lines = parse_lines_no_separator(all_lines, elements);

        let expected: u64 = 1731345124;

        assert_eq!(parse_number_no_separator(&lines[0]), Some(expected));
    }

    #[test]
    fn hostname() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/sys/kernel/hostname");

        let lines = read_lines(path).expect("Could not read");

        let info = parse_lines(lines, vec![], true);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0], "some-name");
    }

    #[test]
    fn version() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/version");

        let lines = read_lines(path).expect("Could not read");

        let info = parse_lines(lines, vec![], true);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0], "Linux version 4.14.44-gafd0c90dd7be (jenkins@miro) (gcc version 4.7 (GCC)) #1 SMP Wed Jul 19 11:56:13 CEST 2023");
    }

    #[test]
    fn cpuinfo_with_key() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/cpuinfo");

        let lines = read_lines(path).expect("Could not read");
        let elements = vec![("model name", false)];

        let info = parse_lines(lines, elements, false);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0], "model name : ARMv7 Processor rev 10 (v7l)");
    }

    #[test]
    fn cpuinfo_without_key() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/cpuinfo");

        let lines = read_lines(path).expect("Could not read");
        let elements = vec![("model name", false)];

        let info = parse_lines(lines, elements, true);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0], "ARMv7 Processor rev 10 (v7l)");
    }

    #[test]
    fn mem_size() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/meminfo");

        let all_lines = read_lines(path).expect("Could not read");
        let elements = vec![("MemTotal:", false)];
        let lines = parse_lines(all_lines, elements, true);

        let expected: u32 = 990180;

        assert_eq!(parse_number(&lines[0]).ok(), Some(expected));
    }

    #[test]
    fn new_ok() {
        assert!(LinuxMachine::new().is_ok());
    }

    /*
    #[test]
    fn parse_network_info_ok() {
        let input_route_info = "default via 192.168.42.1 dev eth0 proto dhcp src 192.168.42.105 metric 100\n192.168.42.0/24 dev eth0 proto kernel scope link src 192.168.42.105 metric 100";
        let input_addres_info = "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc pfifo_fast state UP group default qlen 1000\n    link/ether b8:27:eb:10:c6:85 brd ff:ff:ff:ff:ff:ff\n    inet 192.168.42.105/24 brd 192.168.42.255 scope global dynamic noprefixroute eth0\n       valid_lft 69343sec preferred_lft 69343sec\n    inet6 fe80::bbdd:4dac:bd7c:d839/64 scope link noprefixroute\n       valid_lft forever preferred_lft forever";

        let mut expected = pw::messages::NetworkInterface::default();
        expected.name = "eth0".to_string();
        expected.set_proto(pw::messages::network_interface::Rtpproto::Dhcp);
        expected.mac = mac_from_string("b8:27:eb:10:c6:85");
        expected.ipv4 = ip_from_string("192.168.42.105");
        expected.subnet_mask = parse_number("24").unwrap();
        expected.gateway = ip_from_string("192.168.42.1");
        expected.broadcast = ip_from_string("192.168.42.255");

        assert_eq!(parse_network_info(), Some(expected));
    }
    */
}
