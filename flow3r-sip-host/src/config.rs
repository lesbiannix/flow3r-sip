use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Deserialize, Clone)]
pub struct SipConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub extension: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RtpConfig {
    pub local_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub sip: SipConfig,
    pub rtp: RtpConfig,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        let mut cfg: Config = toml::from_str(&data)?;

        // Environment variable overrides
        if let Ok(server) = env::var("EVENTPHONE_HOST") {
            cfg.sip.server = server;
        }
        if let Ok(user) = env::var("EVENTPHONE_USER") {
            cfg.sip.username = user;
        }
        if let Ok(pass) = env::var("EVENTPHONE_PASS") {
            cfg.sip.password = pass;
        }
        if let Ok(ext) = env::var("EVENTPHONE_EXTENSION") {
            cfg.sip.extension = ext;
        }
        if let Ok(port) = env::var("EVENTPHONE_RTP_PORT") {
            cfg.rtp.local_port = port.parse().unwrap_or(cfg.rtp.local_port);
        }

        // Validate port ranges
        if !(1..=65535).contains(&cfg.sip.port) {
            return Err(anyhow::anyhow!("SIP port out of range: {}", cfg.sip.port));
        }
        if !(1..=65535).contains(&cfg.rtp.local_port) {
            return Err(anyhow::anyhow!(
                "RTP port out of range: {}",
                cfg.rtp.local_port
            ));
        }

        Ok(cfg)
    }
}
