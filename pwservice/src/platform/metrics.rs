use super::{
    error::PlatformError,
    utils::{
        parse_cpu_stat_lines, parse_lines_as_numbers, parse_lines_no_separator, parse_number,
        parse_number_no_separator, read_lines,
    },
};

impl Metrics {
    pub fn new() -> Result<Self, PlatformError> {
        let metrics = load();
        Ok(Self { metrics })
    }

    pub fn serialize(&self) -> Vec<u8> {
        common::serialize_metrics(&self.metrics)
    }
}

/// Struct that encapsulates data.
pub struct Metrics {
    metrics: common::pw::messages::Metrics,
}

fn load() -> common::pw::messages::Metrics {
    let mut metrics = common::pw::messages::Metrics::default();

    if let Some(no_of_processes) = parse_no_of_processes() {
        if no_of_processes.len() == 3 {
            metrics.processes = no_of_processes[0];
            metrics.runnable_processes = no_of_processes[1];
            metrics.processes_blocked = no_of_processes[2];
        }
    }

    if let Some(values) = parse_meminfo() {
        if values.len() == 5 {
            let info = common::pw::messages::MemInfo {
                mem_total: values[0],
                mem_free: values[1],
                mem_available: values[2],
                swap_total: values[3],
                swap_free: values[4],
            };

            metrics.mem_info = Some(info);
        }
    }

    if let Some(values) = parse_cpuload() {
        if values.len() > 1 {
            let mut cpus: Vec<common::pw::messages::CpuLoad> = vec![];

            for value in &values {
                let cpu = common::pw::messages::CpuLoad {
                    user: value[0],
                    nice: value[1],
                    system: value[2],
                    idle: value[3],
                    iowait: value[4],
                };
                cpus.push(cpu);
            }

            metrics.cpu_load = cpus;
        }
    }

    metrics
}

/// Read no of procceses from stat.
fn parse_no_of_processes() -> Option<Vec<u32>> {
    if let Ok(all_lines) = read_lines("/proc/stat") {
        let elements = vec![
            ("processes", false),
            ("procs_running", false),
            ("procs_blocked", false),
        ];
        let length = elements.len();

        let lines = parse_lines_no_separator(all_lines, elements);

        let mut values: Vec<u32> = vec![];
        for line in &lines {
            if let Some(value) = parse_number_no_separator(line) {
                values.push(value);
            }
        }

        if values.len() == length {
            return Some(values);
        }
    }

    None
}

/// Read desired fields from meminfo.
fn parse_meminfo() -> Option<Vec<u32>> {
    if let Ok(lines) = read_lines("/proc/meminfo") {
        let elements = vec![
            ("MemTotal", true),
            ("MemFree", true),
            ("MemAvailable", true),
            ("SwapTotal", true),
            ("SwapFree", true),
        ];
        let length = elements.len();
        let values = parse_lines_as_numbers(lines, elements, true);

        if values.len() == length {
            return Some(values);
        }
    }

    None
}

/// Parse usage stat pr cpu from stat.
fn parse_cpuload() -> Option<Vec<Vec<u64>>> {
    if let Ok(lines) = read_lines("/proc/stat") {
        let values = parse_cpu_stat_lines(lines);
        if values.len() > 0 {
            return Some(values);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, vec};

    use crate::platform::utils::{parse_cpu_stat_lines, parse_lines_as_numbers};

    use super::*;

    #[test]
    fn parse_no_of_processes_ok() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/stat");

        let all_lines = read_lines(path).expect("Could not read");
        let elements = vec![
            ("processes", false),
            ("procs_running", false),
            ("procs_blocked", false),
        ];

        let lines = parse_lines_no_separator(all_lines, elements);

        let mut values: Vec<u32> = vec![];
        for line in &lines {
            if let Some(value) = parse_number_no_separator(line) {
                values.push(value);
            }
        }

        let expected: Vec<u32> = vec![7247, 2, 1];

        assert_eq!(values, expected);
    }

    #[test]
    fn parse_meminfo_ok() {
        {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("resources/test/proc/meminfo1");

            let lines = read_lines(path).expect("Could not read");
            let elements = vec![
                ("MemTotal", false),
                ("MemFree", false),
                ("MemAvailable", false),
                ("SwapTotal", false),
                ("SwapFree", false),
            ];

            let values: Vec<u32> = parse_lines_as_numbers(lines, elements, true);
            let expected: Vec<u32> = vec![990180, 934760, 940044, 0, 0];

            assert_eq!(values.len(), expected.len());
            assert_eq!(values, expected);
        }

        {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("resources/test/proc/meminfo2");

            let lines = read_lines(path).expect("Could not read");
            let elements = vec![
                ("MemTotal", false),
                ("MemFree", false),
                ("MemAvailable", false),
                ("SwapTotal", false),
                ("SwapFree", false),
            ];

            let values: Vec<u32> = parse_lines_as_numbers(lines, elements, true);
            let expected: Vec<u32> = vec![16112208, 6175036, 10443152, 4194300, 2067736];

            assert_eq!(values.len(), expected.len());
            assert_eq!(values, expected);
        }
    }

    #[test]
    fn parse_meminfo_wrong_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/cpuinfo1");

        let lines = read_lines(path).expect("Could not read");
        let elements = vec![
            ("MemTotal", false),
            ("MemFree", false),
            ("MemAvailable", false),
            ("SwapTotal", false),
            ("SwapFree", false),
        ];

        let values: Vec<u32> = parse_lines_as_numbers(lines, elements, true);
        let expected: Vec<u32> = vec![];

        assert_eq!(values.len(), expected.len());
        assert_eq!(values, expected);
    }

    #[test]
    fn parse_cpu_stat_lines_ok() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/proc/stat");

        let lines = read_lines(path).expect("Could not read");
        let values = parse_cpu_stat_lines(lines);

        let expected: Vec<Vec<u64>> = vec![
            vec![23192, 7901, 77014, 761012, 33503],
            vec![3018, 1237, 11821, 91831, 4828],
            vec![3280, 994, 9937, 94212, 4422],
            vec![2385, 1130, 10107, 94505, 4450],
            vec![2858, 1031, 9938, 94869, 4189],
            vec![2858, 774, 8283, 97220, 3670],
            vec![2820, 905, 9100, 96179, 3966],
            vec![3056, 1024, 9349, 95230, 4191],
            vec![2914, 802, 8475, 96962, 3783],
        ];

        assert_eq!(values, expected);
    }
}
