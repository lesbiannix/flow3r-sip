use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct SipConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub sip: SipConfig,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&data)?;
        Ok(cfg)
    }
}
