use serde::{Deserialize, Serialize};

pub struct SatState {
    mode: Mode,
    reboot_count: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Mode {
    NOMINAL,
    DEGRADED,
    SAFE,
}

impl SatState {
    pub fn new() -> Self {
        SatState {
            mode: Mode::SAFE,
            reboot_count: 0,
        }
    }
}
