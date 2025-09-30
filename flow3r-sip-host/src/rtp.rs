use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;


    let mut s = sample;
    let sign = if s < 0 { 0x80 } else { 0 };
    if s < 0 {
        s = -s
    };
    let mut u = ((s >> 3) & 0x1F) as u8;
    u |= 0x70;
    u ^= sign ^ 0x55;
    u
}

#[allow(dead_code)]
#[allow(dead_code)]
pub async fn send_rtp(
    socket: &Arc<UdpSocket>,
    target: &SocketAddr,
    seq: u16,
    ts: u32,
    ssrc: u32,
    payload: &[u8],
) -> anyhow::Result<()> {
    let mut pkt = Vec::with_capacity(12 + payload.len());
    pkt.push(0x80); // Version 2
    pkt.push(0x00); // Payload type 0 (PCMU)
    pkt.extend_from_slice(&seq.to_be_bytes());
    pkt.extend_from_slice(&ts.to_be_bytes());
    pkt.extend_from_slice(&ssrc.to_be_bytes());
    pkt.extend_from_slice(payload);
    socket.send_to(&pkt, target).await?;
    Ok(())
}
