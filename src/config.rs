use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mode: Mode,
    pub server: Option<ServerConfig>,
    pub client: Option<ClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Server,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_ip: String,
    pub listen_port: u16,
    pub backend_ip: String,
    pub backend_port: u16,
    pub connection_pool_size: usize,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub local_listen_ip: String,
    pub local_listen_port: u16,
    pub server_ip: String,
    pub server_port: u16,
    pub connection_pool_size: usize,
    pub buffer_size: usize,
    pub enable_zero_rtt: bool,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.mode {
            Mode::Server => {
                if self.server.is_none() {
                    return Err("Server mode requires server configuration".to_string());
                }
            }
            Mode::Client => {
                if self.client.is_none() {
                    return Err("Client mode requires client configuration".to_string());
                }
            }
        }
        Ok(())
    }
}
