use super::utils::{find_default_dev, find_iface_info, parse_lines, parse_lines_no_separator, parse_number, parse_number_no_separator, read_lines};

pub fn load() -> common::pw::messages::Machine {
    let mut machine = common::pw::messages::Machine::default();

    if let Some(value) = parse_mac_and_ip() {
        (machine.mac, machine.ipv4) = value;
    }

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

    machine
}

fn parse_mac_and_ip() -> Option<(u64, u32)> {
    if let Some(dev) = find_default_dev() {
        return find_iface_info(&dev);
    }

    None
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
        let elements = vec![ ("model name", false) ];
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
}
