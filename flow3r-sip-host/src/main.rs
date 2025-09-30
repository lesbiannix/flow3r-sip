mod config;
mod rtp;
mod sip;

use std::sync::Arc;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::Config::load("config.toml")?;

    // Bind local UDP
    let local_addr: std::net::SocketAddr = format!("0.0.0.0:{}", cfg.sip.port).parse()?;
    let socket = Arc::new(UdpSocket::bind(local_addr).await?);
    println!("Bound local UDP socket to {}", local_addr);

    // Resolve SIP server
    let mut server_iter = tokio::net::lookup_host((cfg.sip.server.trim(), cfg.sip.port)).await?;
    let sip_server = server_iter
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve SIP server hostname"))?;

    // Spawn listener
    sip::spawn_listener(Arc::clone(&socket));

    // Send REGISTER
    let rtp_target =
        sip::register(&socket, &sip_server, &cfg.sip.username, &cfg.sip.password).await?;
    if let Some(target) = rtp_target {
        println!("RTP target: {}", target);
    } else {
        println!("No RTP target received yet.");
    }

    Ok(())
}
