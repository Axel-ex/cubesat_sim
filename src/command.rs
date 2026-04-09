use crate::state::Mode;
use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    #[serde(rename = "camera.capture")]
    CameraCapture { id: u64 },

    #[serde(rename = "camera.on")]
    CameraTurnOn { id: u64 },

    #[serde(rename = "camera.off")]
    CameraTurnOff { id: u64 },

    #[serde(rename = "eps.read_main_voltage")]
    EpsReadMainVoltage { id: u64 },

    #[serde(rename = "health.get")]
    HealthGet { id: u64 },

    #[serde(rename = "mode.set")]
    ModeSet { id: u64, mode: Mode },
}

pub fn parse_cmd(cmd_string: &str) -> Result<Command> {
    Ok(serde_json::from_str::<Command>(cmd_string)?)
}

pub async fn dispatch_cmd(cmd: Command, cmd_sender: &mpsc::Sender<Command>) {
    if let Err(e) = cmd_sender.send(cmd).await {
        error!("Command channel send failed: {}", e);
    }
}
