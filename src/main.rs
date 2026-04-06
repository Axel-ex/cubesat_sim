use cubesat_sim::tmtc::tmtc_task;
use tokio::sync::mpsc::channel;

use anyhow::Result;
use cubesat_sim::state::SatState;
use cubesat_sim::subsystems::coms::radio_task;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Cubesat initialising...");
    //tmtc task receive raw byte, form a command and dispatch it
    let (raw_cmd_sender, raw_cmd_rcvr) = channel(1000);
    // tmtc task receive telemetry event from from susystems (senders)
    let (telemetry_sender, telemetry_rcvr) = channel(1000);
    //subsystems receive command from tmtc task
    let (cmd_sender, cmd_rcvr) = channel(1000);

    tokio::task::spawn(radio_task(raw_cmd_sender)).await??;
    tokio::task::spawn(tmtc_task(raw_cmd_rcvr, cmd_sender, telemetry_rcvr));

    let mut state = SatState::new();
    loop {}
}
