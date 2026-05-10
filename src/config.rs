use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::user::User;

const CONFIG_DIR: &str = "/etc/quick-node";
const CONFIG_PATH: &str = "/etc/quick-node/config.json";
const SINGBOX_CONFIG_PATH: &str = "/etc/quick-node/singbox-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_ip: String,
    pub hy_port: u16,
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

    pub fn generate_singbox_config(&self, users: &[User]) -> Result<()> {
        let user_list: Vec<serde_json::Value> = users
            .iter()
            .filter(|u| u.enabled)
            .map(|u| json!({"name": u.name, "password": u.password}))
            .collect();

        let config = json!({
            "inbounds": [{
                "type": "hysteria2",
                "tag": "hy2-in",
                "listen": "::",
                "listen_port": self.hy_port,
                "users": user_list,
                "tls": {
                    "enabled": true,
                    "certificate_path": format!("{}/cert.pem", CONFIG_DIR),
                    "key_path": format!("{}/key.pem", CONFIG_DIR)
                }
            }],
            "outbounds": [{"type": "direct", "tag": "direct"}]
        });

        std::fs::write(SINGBOX_CONFIG_PATH, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }
}
