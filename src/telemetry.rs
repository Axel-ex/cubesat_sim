use crate::command::Command;
use crate::state::Mode;
use anyhow::Result;
use log::error;
use serde::Serialize;
use tokio::sync::mpsc;

// TELEMETRY
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum TelemetryEvent {
    #[serde(rename = "tc_response")]
    TcResponse { response_code: u32 },

    #[serde(rename = "health")]
    Health {
        mode: Mode,
        camera_fail_streak: u32,
        i2c_timeouts: u32,
    },

    #[serde(rename = "log")]
    Log { level: String, msg: String },
}

pub fn serialize_telemetry(evt: &TelemetryEvent) -> Result<String> {
    Ok(serde_json::to_string(evt)?)
}

pub async fn dispatch_cmd(cmd: Command, cmd_sender: &mpsc::Sender<Command>) {
    if let Err(e) = cmd_sender.send(cmd).await {
        error!("Command channel send failed: {}", e);
    }
}
