use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::user::User;

const CONFIG_DIR: &str = "/etc/quick-vless";
const CONFIG_PATH: &str = "/etc/quick-vless/config.json";
const XRAY_CONFIG_PATH: &str = "/etc/quick-vless/xray-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_ip: String,
    pub vless_port: u16,
    pub private_key: String,
    pub public_key: String,
    pub short_id: String,
    pub server_name: String,
    pub xray_api_addr: String,
    pub sub_port: u16,
    pub socks_port: u16,
}

impl AppConfig {
    pub fn config_dir() -> &'static str {
        CONFIG_DIR
    }

    pub fn load() -> Result<Self> {
        let data = std::fs::read_to_string(CONFIG_PATH)
            .context("Failed to read config. Run 'quick-vless init' first.")?;
        serde_json::from_str(&data).context("Invalid config.json")
    }

    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(CONFIG_DIR)?;
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(CONFIG_PATH, data)?;
        Ok(())
    }

    pub fn generate_xray_config(&self, users: &[User]) -> Result<()> {
        let enabled_users: Vec<&User> = users.iter().filter(|u| u.enabled).collect();

        let vless_clients: Vec<serde_json::Value> = enabled_users
            .iter()
            .map(|u| {
                json!({
                    "id": u.uuid.to_string(),
                    "email": u.email,
                    "flow": "xtls-rprx-vision"
                })
            })
            .collect();

        let socks_accounts: Vec<serde_json::Value> = enabled_users
            .iter()
            .map(|u| {
                json!({
                    "user": u.name,
                    "pass": u.socks_pass
                })
            })
            .collect();

        let config = json!({
            "log": {
                "loglevel": "warning"
            },
            "stats": {},
            "api": {
                "tag": "api",
                "services": ["StatsService"]
            },
            "policy": {
                "levels": {
                    "0": {
                        "statsUserUplink": true,
                        "statsUserDownlink": true
                    }
                },
                "system": {
                    "statsInboundUplink": true,
                    "statsInboundDownlink": true
                }
            },
            "inbounds": [
                {
                    "tag": "vless-reality",
                    "port": self.vless_port,
                    "protocol": "vless",
                    "settings": {
                        "clients": vless_clients,
                        "decryption": "none"
                    },
                    "streamSettings": {
                        "network": "tcp",
                        "security": "reality",
                        "realitySettings": {
                            "show": false,
                            "dest": format!("{}:443", self.server_name),
                            "xver": 0,
                            "serverNames": [self.server_name],
                            "privateKey": self.private_key,
                            "shortIds": [self.short_id]
                        }
                    },
                    "sniffing": {
                        "enabled": true,
                        "destOverride": ["http", "tls"]
                    }
                },
                {
                    "tag": "socks-in",
                    "port": self.socks_port,
                    "protocol": "socks",
                    "settings": {
                        "auth": "password",
                        "accounts": socks_accounts,
                        "udp": true
                    }
                },
                {
                    "tag": "api-in",
                    "listen": "127.0.0.1",
                    "port": self.xray_api_addr.split(':').last()
                        .unwrap_or("10085").parse::<u16>().unwrap_or(10085),
                    "protocol": "dokodemo-door",
                    "settings": {
                        "address": "127.0.0.1"
                    }
                }
            ],
            "outbounds": [
                {
                    "tag": "direct",
                    "protocol": "freedom"
                }
            ],
            "routing": {
                "rules": [
                    {
                        "type": "field",
                        "inboundTag": ["api-in"],
                        "outboundTag": "api"
                    }
                ]
            }
        });

        std::fs::write(XRAY_CONFIG_PATH, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }
}
