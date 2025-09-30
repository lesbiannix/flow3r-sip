use anyhow::Result;
use md5;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// Register with Eventphone and return RTP target
    let branch = "z9hG4bK-1";
    let call_id = "1001@flow3r";
    let cseq = 1;
    let contact = format!("<sip:{}@0.0.0.0:5060>", username);

    // Initial REGISTER
    let mut register_msg = format!(
        "REGISTER sip:{} SIP/2.0\r\nVia: SIP/2.0/UDP 0.0.0.0:5060;branch={}\r\nFrom: <sip:{}@{}>;tag=1\r\nTo: <sip:{}@{}>\r\nCall-ID: {}\r\nCSeq: {} REGISTER\r\nContact: {}\r\nContent-Length: 0\r\n\r\n",
        server, branch, username, server, username, server, call_id, cseq, contact
    );
    socket.send_to(register_msg.as_bytes(), server).await?;
    println!("Initial REGISTER sent to {}", server);

    // Wait 401
    let mut buf = [0u8; 1500];
    let (len, _) = socket.recv_from(&mut buf).await?;
    let response = String::from_utf8_lossy(&buf[..len]);

    if let Some(www_auth) = parse_www_authenticate(&response) {
        let digest = compute_digest(
            username,
            password,
            "REGISTER",
            &format!("sip:{}", server),
            &www_auth,
        );
        register_msg = format!(
            "REGISTER sip:{} SIP/2.0\r\nVia: SIP/2.0/UDP 0.0.0.0:5060;branch={}\r\nFrom: <sip:{}@{}>;tag=1\r\nTo: <sip:{}@{}>\r\nCall-ID: {}\r\nCSeq: {} REGISTER\r\nContact: {}\r\nAuthorization: {}\r\nContent-Length: 0\r\n\r\n",
            server, branch, username, server, username, server, call_id, cseq, contact, digest
        );
        socket.send_to(register_msg.as_bytes(), server).await?;
        println!("REGISTER with digest sent to {}", server);
    }

    // Wait 200 OK with SDP
    let (len, _) = socket.recv_from(&mut buf).await?;
    let response = String::from_utf8_lossy(&buf[..len]);
    if let Some(sdp) = extract_sdp(&response) {
        if let Some(rtp_target) = parse_sdp_rtp(&sdp) {
            println!("Parsed RTP target from SDP: {}", rtp_target);
            return Ok(Some(rtp_target));
        }
    }

    Ok(None)
}

/// Extract WWW-Authenticate header

    for line in response.lines() {
        if line.to_lowercase().starts_with("www-authenticate:") {
            return Some(line["WWW-Authenticate:".len()..].trim().to_string());
        }
    }
    None
}

/// Compute SIP digest
fn compute_digest(
    username: &str,
    password: &str,
    method: &str,
    uri: &str,
    www_auth: &str,
) -> String {
    let realm = extract_field(www_auth, "realm").unwrap_or_else(|| "".to_string());
    let nonce = extract_field(www_auth, "nonce").unwrap_or_else(|| "".to_string());

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

/// Extract value from header
fn extract_field(header: &str, field: &str) -> Option<String> {
    let search = format!("{}=\"", field);
    let start = header.find(&search)? + search.len();
    let end = header[start..].find('"')? + start;
    Some(header[start..end].to_string())
}


    response.split("\r\n\r\n").nth(1)
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

/// Spawn SIP listener
#[allow(dead_code)]
#[allow(dead_code)]
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
