use std::io;
use serde::{Deserialize, Serialize};
use rmp_serde::Serializer;
use crate::data_types::{traits, utils};
use super::utils::{find_default_dev, find_iface_info, parse_lines, read_lines};

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Machine {
    pub mac: u64,
    pub ipv4: u32,
    pub uptime: u32,
    pub boot_id: String,
    pub version: String,
    pub cpuinfo: Vec<String>,
    pub meminfo: Vec<String>,
}

impl Machine {    
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load(&mut self) {
        (self.mac, self.ipv4) = Machine::parse_mac_and_ip();
        self.uptime = Machine::parse_uptime();
        self.boot_id = Machine::parse_boot_id();
        self.version = Machine::parse_version();
        self.cpuinfo = Machine::parse_cpuinfo();
        self.meminfo = Machine::parse_meminfo();
    }

    fn parse_mac_and_ip() -> (u64, u32) {
        let dev = find_default_dev();
        if dev.is_some() {
            let info = find_iface_info(&dev.unwrap());
            if info.is_some() {
                return info.unwrap();
            }
        }
        
        (0,0)
    }

    fn parse_uptime() -> u32 {
        let lines = read_lines("/proc/uptime").expect("Could not read /proc/uptime");
        if !lines.is_empty() {
            // /proc/uptime contains '2747.41 17969.77', where first number is seconds since boot.
            let floats = lines[0].split(" ").filter_map(|s| s.parse::<f32>().ok()).collect::<Vec<_>>();
            if !floats.is_empty() {
                return floats[0] as u32;
            }   
        }
        
        0
    }

    fn parse_boot_id() -> String {
        let lines = read_lines("/proc/sys/kernel/random/boot_id").expect("Could not read /proc/sys/kernel/random/boot_id");
        parse_lines(lines, vec![]).pop().unwrap_or("".to_string())
    }

    fn parse_version() -> String {
        let lines = read_lines("/proc/version").expect("Could not read /proc/version");
        parse_lines(lines, vec![]).pop().unwrap_or("".to_string())
    }

    fn parse_cpuinfo() -> Vec<String> {
        let lines = read_lines("/proc/cpuinfo").expect("Could not read /proc/cpuinfo");
        let elements = vec![ ("vendor_id", false), ("model name", false), ("cpu cores", false), ("cpu MHz", false) ];
        
        parse_lines(lines, elements)
    }

    fn parse_meminfo() -> Vec<String> {
        let lines = read_lines("/proc/meminfo").expect("Could not read /proc/meminfo");
        let elements = vec![ ("MemTotal:", false), ("MemFree:", false), ("MemAvailable:", false) ];
        
        parse_lines(lines, elements)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    
    #[test]
    fn version() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/version");

        let lines = read_lines(path).expect("Could not read");

        let info = parse_lines(lines, vec![]);
        assert_eq!(info.len(), 1);
            assert_eq!(info[0], "Linux version 4.14.44-gafd0c90dd7be (jenkins@miro) (gcc version 4.7 (GCC)) #1 SMP Wed Jul 19 11:56:13 CEST 2023");
    }

    #[test]
    fn cpuinfo() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/cpuinfo");
    
        let lines = read_lines(path).expect("Could not read");
        let elements = vec![ ("model name", false) ];
        
        let info = parse_lines(lines, elements);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0], "model name : ARMv7 Processor rev 10 (v7l)");
    }

    #[test]
    fn meminfo() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/meminfo");
    
        let lines = read_lines(path).expect("Could not read");
        let elements = vec![ ("MemTotal:", false), ("MemFree:", false), ("MemAvailable:", false) ];
        
        let info = parse_lines(lines, elements);
        assert_eq!(info.len(), 3);
        assert_eq!(info[0], "MemTotal: 990180 kB");
        assert_eq!(info[1], "MemFree: 934760 kB");
        assert_eq!(info[2], "MemAvailable: 940044 kB");
    }
}
