use crate::command::{dispatch_cmd, parse_cmd, Command};
use crate::telemetry::{serialize_telemetry, TelemetryEvent};
use anyhow::Result;
use log::{error, info};
use tokio::sync::mpsc;

//
// ===== TM/TC task =====
//
// cmd_rcv: raw command lines from radio (already line-framed)
// cmd_sender: parsed commands forwarded to the system
// telemetry_rcv: typed telemetry events coming from the system
pub async fn tmtc_task(
    mut raw_cmd_rcvr: mpsc::Receiver<String>,
    cmd_sender: mpsc::Sender<Command>,
    mut telemetry_rcvr: mpsc::Receiver<TelemetryEvent>,
    raw_telemetry_sender: mpsc::Sender<String>,
) -> Result<()> {
    loop {
        tokio::select! {
            maybe_cmd = raw_cmd_rcvr.recv() => {
                let Some(cmd_string) = maybe_cmd else {
                    // command input closed
                    break;
                };

                match parse_cmd(cmd_string.trim()) {
                    Ok(cmd) => dispatch_cmd(cmd, &cmd_sender).await,
                    Err(e) => error!("Error parsing command: {} | input={}", e, cmd_string),
                }
            }

            maybe_evt = telemetry_rcvr.recv() => {
                let Some(evt) = maybe_evt else {
                    // telemetry input closed
                    break;
                };

                match serialize_telemetry(&evt) {
                    Ok(line) => {
                        info!("{:?}", line);
                        // serialize and send to the radio for downlink
                        todo!()
                    }
                    Err(e) => error!("Telemetry serialization error: {}", e),
                }
            }
        }
    }

    Ok(())
}
