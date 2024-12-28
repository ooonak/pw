use crate::platform::utils::parse_lines_as_numbers;

use super::{
    error::PlatformError,
    utils::read_lines,
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

    if let Some(values) = parse_meminfo() {
        if values.len() == 5 {
            let info = common::pw::messages::MemInfo {
            mem_total: values[0],
            mem_free: values[1],
            mem_available: values[2],
            swap_total: values[3],
            swap_free: values[4] };

            metrics.mem_info = Some(info);
        }
    }

    metrics
}

/// Read desired fields from meminfo
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
        let lines = parse_lines_as_numbers(lines, elements, true);

        if lines.len() == length {
            return Some(lines);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, vec};

    use crate::platform::utils::parse_lines_as_numbers;

    use super::*;

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

            let values = parse_lines_as_numbers(lines, elements, true);
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

            let values = parse_lines_as_numbers(lines, elements, true);
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

        let values = parse_lines_as_numbers(lines, elements, true);
        let expected: Vec<u32> = vec![];

        assert_eq!(values.len(), expected.len());
        assert_eq!(values, expected);
    }
}
