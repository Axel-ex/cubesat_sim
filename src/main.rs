use std::time::Duration;

use anyhow::Result;
use cubesat_sim::subsystems::coms::radio_task;
use log::info;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Cubesat initialising...");
    sleep(Duration::from_secs(1)).await;
    tokio::task::spawn(radio_task()).await??;

    Ok(())
}
