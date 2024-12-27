use super::error::PlatformError;

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

    //todo!();

    metrics
}
