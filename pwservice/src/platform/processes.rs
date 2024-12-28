use super::error::PlatformError;

impl Processes {
    pub fn new() -> Result<Self, PlatformError> {
        let processes = load();
        Ok(Self { processes })
    }

    pub fn serialize(&self) -> Vec<u8> {
        common::serialize_process(&self.processes)
    }
}

/// Struct that encapsulates data.
pub struct Processes {
    processes: common::pw::messages::Processes,
}

fn load() -> common::pw::messages::Processes {
    let mut processes = common::pw::messages::Processes::default();

    //todo!();

    processes
}
