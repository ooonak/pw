use serde::{Deserialize, Serialize};
use rmp_serde::Serializer;
use crate::data_types::{traits, utils};

use super::utils::read_line;

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Machine {
    pub boot_id: String,
}

impl Machine {    
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load(&mut self) {
        self.boot_id = read_line(std::path::Path::new("/proc/sys/kernel/random/boot_id")).expect("Could not load boot id");
    }
}

impl traits::Message for Machine {
}
