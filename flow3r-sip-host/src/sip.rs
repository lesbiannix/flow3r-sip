use anyhow::Result;
use hex;
use md5;
use rand::RngCore;
use rand::rngs::ThreadRng;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// Simple WWW-Authenticate container
#[derive(Debug, Default)]
pub struct WwwAuth {
    pub realm: Option<String>,
    pub nonce: Option<String>,
    pub opaque: Option<String>,
    pub algorithm: Option<String>,
    pub qop: Option<String>,
}

/// Parse WWW-Authenticate header
pub fn parse_www_authenticate_fields(response: &str) -> Option<WwwAuth> {
    for line in response.lines() {
        if line.to_lowercase().starts_with("www-authenticate:") {
            let rest = line["www-authenticate:".len()..].trim();
            let rest = rest.strip_prefix("Digest").unwrap_or(rest).trim();
            let mut out = WwwAuth::default();
            for part in rest.split(',') {
                let part = part.trim();
                if let Some(eq) = part.find('=') {
                    let key = part[..eq].trim().to_lowercase();
                    let mut value = part[eq + 1..].trim().to_string();
                    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                        value = value[1..value.len() - 1].to_string();
                    }
                    match key.as_str() {
                        "realm" => out.realm = Some(value),
                        "nonce" => out.nonce = Some(value),
                        "opaque" => out.opaque = Some(value),
                        "algorithm" => out.algorithm = Some(value),
                        "qop" => out.qop = Some(value),
                        _ => {}
                    }
                }
            }
            return Some(out);
        }
    }
    None
}

/// Generate a simple random cnonce
fn generate_cnonce(rng: &mut ThreadRng) -> String {
    let mut b = [0u8; 12];
    rng.fill_bytes(&mut b);
    hex::encode(b)
}

/// Compute digest for REGISTER
fn compute_digest(
    username: &str,
    password: &str,
    method: &str,
    uri: &str,
    auth: &WwwAuth,
) -> String {
    let realm = auth.realm.as_deref().unwrap_or("");
    let nonce = auth.nonce.as_deref().unwrap_or("");
    let ha1 = format!(
        "{:x}",
        md5::compute(format!("{}:{}:{}", username, realm, password))
    );
    let ha2 = format!("{:x}", md5::compute(format!("{}:{}", method, uri)));
    let response = format!("{:x}", md5::compute(format!("{}:{}:{}", ha1, nonce, ha2)));

    format!(
        "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", response=\"{}\"",
        username, realm, nonce, uri, response
    )
}

/// Build REGISTER SIP message
fn build_register(
    username: &str,
    server: &str,
    branch: &str,
    call_id: &str,
    cseq: u32,
    auth_header: Option<&str>,
) -> String {
    let contact = format!("<sip:{}@0.0.0.0:5060>", username);
    let mut msg = format!(
        "REGISTER sip:{} SIP/2.0\r\n\
         Via: SIP/2.0/UDP 0.0.0.0:5060;branch={}\r\n\
         From: <sip:{}@{}>;tag=1\r\n\
         To: <sip:{}@{}>\r\n\
         Call-ID: {}\r\n\
         CSeq: {} REGISTER\r\n\
         Contact: {}\r\n",
        server, branch, username, server, username, server, call_id, cseq, contact
    );
    if let Some(auth) = auth_header {
        msg.push_str(&format!("Authorization: {}\r\n", auth));
    }
    msg.push_str("Content-Length: 0\r\n\r\n");
    msg
}

/// Parse RTP target from SDP
pub fn parse_sdp_rtp(sdp: &str) -> Option<SocketAddr> {
    let mut ip = None;
    let mut port = None;
    for line in sdp.lines() {
        if line.starts_with("c=IN IP4 ") {
            ip = Some(line["c=IN IP4 ".len()..].trim());
        }
        if line.starts_with("m=audio ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                port = parts[1].parse::<u16>().ok();
            }
        }
    }
    match (ip, port) {
        (Some(ip), Some(port)) => ip.parse().ok().map(|ip| SocketAddr::new(ip, port)),
        _ => None,
    }
}

/// Register with SIP server and return optional RTP target
pub async fn register(
    socket: &Arc<UdpSocket>,
    server: &SocketAddr,
    username: &str,
    password: &str,
) -> Result<Option<SocketAddr>> {
    let branch = "z9hG4bK-1";
    let call_id = "1001@flow3r";
    let cseq = 1;

    let register_msg = build_register(username, &server.to_string(), branch, call_id, cseq, None);
    socket.send_to(register_msg.as_bytes(), server).await?;
    println!("Initial REGISTER sent to {}", server);

    let mut buf = [0u8; 1500];
    let (len, _) = socket.recv_from(&mut buf).await?;
    let response = String::from_utf8_lossy(&buf[..len]);
    println!("SIP message from {}:\n{}", server, response);

    if let Some(auth) = parse_www_authenticate_fields(&response) {
        let digest = compute_digest(
            username,
            password,
            "REGISTER",
            &format!("sip:{}", server),
            &auth,
        );
        let register_msg2 = build_register(
            username,
            &server.to_string(),
            branch,
            call_id,
            cseq + 1,
            Some(&digest),
        );
        println!("Sending REGISTER with digest:\n{}", register_msg2);
        socket.send_to(register_msg2.as_bytes(), server).await?;

        let (len, _) = socket.recv_from(&mut buf).await?;
        let response2 = String::from_utf8_lossy(&buf[..len]);
        println!("SIP response after digest REGISTER:\n{}", response2);

        if let Some(sdp) = response2.split("\r\n\r\n").nth(1) {
            if let Some(rtp_target) = parse_sdp_rtp(sdp) {
                return Ok(Some(rtp_target));
            }
        }
    }

    Ok(None)
}

/// Spawn SIP listener
pub fn spawn_listener(socket: Arc<UdpSocket>) {
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1500];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    println!(
                        "SIP message from {}:\n{}",
                        addr,
                        String::from_utf8_lossy(&buf[..len])
                    );
                }
                Err(e) => {
                    eprintln!("SIP receive error: {:?}", e);
                    break;
                }
            }
        }
    });
}
