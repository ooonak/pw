use super::utils::{find_default_dev, find_iface_info, parse_lines, parse_number, read_lines};

pub fn load() -> common::pw::messages::Machine {
    let mut machine = common::pw::messages::Machine::default();
    
    if let Some(value) = parse_mac_and_ip() {
        (machine.mac, machine.ipv4) = value;
    }

    if let Some(value) = parse_uptime() {
        machine.uptime = value;
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
        machine.cpuinfo = value;
    }
    
    if let Some(value) = parse_meminfo() {
        machine.meminfo = value;
    }
    
    if let Some(value) = parse_mem_size() {
        machine.physical_mem_total_kb = value;
    }

    machine
}

fn parse_mac_and_ip() -> Option<(u64, u32)> {
    if let Some(dev) = find_default_dev() {
        return find_iface_info(&dev);
    }

    None
}

fn parse_uptime() -> Option<u32> {
    if let Some(lines) = read_lines("/proc/uptime").ok() {
        if !lines.is_empty() {
            // /proc/uptime contains '2747.41 17969.77', where first number is seconds since boot.
            let floats = lines[0]
                .split(" ")
                .filter_map(|s| s.parse::<f32>().ok())
                .collect::<Vec<_>>();
            if !floats.is_empty() {
                return Some(floats[0] as u32);
            }
        }
    }

    None
}

fn parse_boot_id() -> Option<String> {
    if let Some(lines) = read_lines("/proc/sys/kernel/random/boot_id").ok() {
        return parse_lines(lines, vec![], true).pop();
    }

    None
}

fn parse_hostname() -> Option<String> {
    if let Some(lines) = read_lines("/proc/sys/kernel/hostname").ok() {
        return parse_lines(lines, vec![], true).pop();
    }
    
    None
}

fn parse_version() -> Option<String> {
    if let Some(lines) = read_lines("/proc/version").ok() {
        return parse_lines(lines, vec![], true).pop();
    }
    
    None
}

fn parse_cpuinfo() -> Option<Vec<String>> {
    if let Some(lines) = read_lines("/proc/cpuinfo").ok() {
        let elements = vec![
            ("vendor_id", false),
            ("model name", false),
            ("cpu cores", false),
            ("cpu MHz", false),
        ];
    
        return Some(parse_lines(lines, elements, false));
    }
    
    None
}

fn parse_meminfo() -> Option<Vec<String>> {
    if let Some(lines) = read_lines("/proc/meminfo").ok() {
        let elements = vec![
            ("MemTotal:", false),
            ("MemFree:", false),
            ("MemAvailable:", false),
        ];

        return Some(parse_lines(lines, elements, false));
    }

    None
}

fn parse_mem_size() -> Option<u32> {
    if let Some(all_lines) = read_lines("/proc/meminfo").ok() {
        let elements = vec![ ("MemTotal:", false) ];
        let lines = parse_lines(all_lines, elements, true);

        return parse_number(&lines[0]).ok()
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

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
    fn meminfo() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/meminfo");

        let lines = read_lines(path).expect("Could not read");
        let elements = vec![
            ("MemTotal:", false),
            ("MemFree:", false),
            ("MemAvailable:", false),
        ];

        let info = parse_lines(lines, elements, false);
        assert_eq!(info.len(), 3);
        assert_eq!(info[0], "MemTotal: 990180 kB");
        assert_eq!(info[1], "MemFree: 934760 kB");
        assert_eq!(info[2], "MemAvailable: 940044 kB");
    }

    #[test]
    fn mem_size() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/meminfo");

        let all_lines = read_lines(path).expect("Could not read");
        let elements = vec![ ("MemTotal:", false) ];
        let lines = parse_lines(all_lines, elements, true);
    
        let expected: u32 = 990180;
        
        assert_eq!(parse_number(&lines[0]).ok(), Some(expected));
    }
}
