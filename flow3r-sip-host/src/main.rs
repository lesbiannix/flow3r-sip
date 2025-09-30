mod config;
mod sip;
mod rtp;

use std::sync::Arc;


use tokio::net::UdpSocket;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::Config::load("config.toml")?;

    // Bind local UDP
    let local_addr: std::net::SocketAddr = format!("0.0.0.0:{}", cfg.sip.port).parse()?;
    let socket = Arc::new(UdpSocket::bind(local_addr).await?);
    println!("Bound local UDP socket to {}", local_addr);

    // Resolve SIP server hostname
    let mut server_iter = tokio::net::lookup_host((cfg.sip.server.trim(), cfg.sip.port)).await?;
    let _sip_server = server_iter.next().ok_or_else(|| anyhow::anyhow!("Could not resolve SIP server hostname"))?;
    Ok(())
}
