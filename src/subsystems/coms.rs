//! Simulate Communication system
//!
//! open a socket and listenb for incoming connections. "nc localhost 8080" to pen a connection
//! with the sim
use crate::tmtc::Command;
use anyhow::Result;
use log::{error, info};
use tokio::sync::mpsc;
use tokio::{io::AsyncReadExt, net::TcpListener};

const ADDR: &str = "127.0.0.1:8080";

#[allow(unreachable_code)]
pub async fn radio_task(raw_cmd_sender: mpsc::Sender<String>) -> Result<()> {
    let listener = TcpListener::bind(ADDR).await?;
    info!("Comm system listening at address {}", ADDR);

    loop {
        let mut buf = [0u8; 1024];
        let (mut socket, _) = listener.accept().await?;

        loop {
            match socket.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        info!("End of transmission");
                        break;
                    }
                    info!("received {} bytes", n);
                    let string_buf = std::str::from_utf8(&buf[..n])?;
                    if let Err(e) = raw_cmd_sender.send(string_buf.to_string()).await {
                        println!("{e}");
                    }
                }
                Err(e) => {
                    error!("Fail reading from the socket: {:#?}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}
