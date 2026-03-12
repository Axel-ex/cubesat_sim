// tmtc.rs
//
// Single module with TM/TC plumbing only:
// - receives raw command lines (String) from comms
// - parses into typed Command (serde)
// - forwards Command to the internal command channel
// - receives typed TelemetryEvent from the system
// - serializes to JSON line (String)
// - forwards JSON line to comms for writing
//
// This is intentionally minimal and matches the structure you had.

use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

//
// ===== Command types (input) =====
// Expect newline-delimited JSON like:
// {"id":1,"cmd":"camera.capture"}
// {"id":2,"cmd":"eps.read_main_voltage"}
// {"id":3,"cmd":"health.get"}
// {"id":4,"cmd":"mode.set","mode":"SAFE"}
//

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Mode {
    NOMINAL,
    DEGRADED,
    SAFE,
}

pub fn parse_cmd(cmd_string: &str) -> Result<Command> {
    Ok(serde_json::from_str::<Command>(cmd_string)?)
}

//
// ===== Telemetry types (output) =====
// You can keep it very simple: one response per command, plus optional health snapshots.
// Everything is one outbound stream: JSON line per event.
//

#[derive(Debug, Serialize)]
pub struct TcResponse {
    pub id: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum TelemetryEvent {
    #[serde(rename = "tc_response")]
    TcResponse(TcResponse),

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

//
// ===== Dispatch wiring =====
// Here dispatch_cmd only forwards parsed commands to the internal command handler.
// The "real" dispatch happens elsewhere (camera/eps/fdir tasks).
//

pub async fn dispatch_cmd(cmd: Command, cmd_sender: &mpsc::Sender<Command>) {
    if let Err(e) = cmd_sender.send(cmd).await {
        error!("Command channel send failed: {}", e);
    }
}

//
// ===== TM/TC task =====
//
// cmd_rcv: raw command lines from comms (already line-framed)
// cmd_sender: parsed commands forwarded to the system
// telemetry_rcv: typed telemetry events coming from the system
// telemetry_sender: serialized JSON lines to comms writer
//

pub async fn tmtc_task(
    mut cmd_rcv: mpsc::Receiver<String>,
    cmd_sender: mpsc::Sender<Command>,
    mut telemetry_rcv: mpsc::Receiver<TelemetryEvent>,
    telemetry_sender: mpsc::Sender<String>,
) -> Result<()> {
    loop {
        tokio::select! {
            // inbound commands
            maybe_cmd = cmd_rcv.recv() => {
                let Some(cmd_string) = maybe_cmd else {
                    // command input closed
                    break;
                };

                match parse_cmd(cmd_string.trim()) {
                    Ok(cmd) => dispatch_cmd(cmd, &cmd_sender).await,
                    Err(e) => error!("Error parsing command: {} | input={}", e, cmd_string),
                }
            }

            // outbound telemetry/events
            maybe_evt = telemetry_rcv.recv() => {
                let Some(evt) = maybe_evt else {
                    // telemetry input closed
                    break;
                };

                match serialize_telemetry(&evt) {
                    Ok(line) => {
                        if let Err(e) = telemetry_sender.send(line).await {
                            error!("Telemetry channel send failed: {}", e);
                        }
                    }
                    Err(e) => error!("Telemetry serialization error: {}", e),
                }
            }
        }
    }

    Ok(())
}
