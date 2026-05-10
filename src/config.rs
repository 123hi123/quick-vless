use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::user::User;

const CONFIG_DIR: &str = "/etc/quick-node";
const CONFIG_PATH: &str = "/etc/quick-node/config.json";
const SS_CONFIG_PATH: &str = "/etc/quick-node/ss-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_ip: String,
    pub ss_port: u16,
    pub server_key: String,
    pub sub_port: u16,
}

impl AppConfig {
    pub fn config_dir() -> &'static str {
        CONFIG_DIR
    }

    pub fn load() -> Result<Self> {
        let data = std::fs::read_to_string(CONFIG_PATH)
            .context("Failed to read config. Run 'quick-node init' first.")?;
        serde_json::from_str(&data).context("Invalid config.json")
    }

    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(CONFIG_DIR)?;
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(CONFIG_PATH, data)?;
        Ok(())
    }

    pub fn generate_ss_config(&self, users: &[User]) -> Result<()> {
        let enabled_users: Vec<serde_json::Value> = users
            .iter()
            .filter(|u| u.enabled)
            .map(|u| {
                json!({
                    "name": u.name,
                    "password": u.ss_key
                })
            })
            .collect();

        let config = json!({
            "server": "0.0.0.0",
            "server_port": self.ss_port,
            "method": "2022-blake3-aes-256-gcm",
            "password": self.server_key,
            "users": enabled_users
        });

        std::fs::write(SS_CONFIG_PATH, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }
}
