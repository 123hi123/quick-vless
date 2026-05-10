use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ss;

const USERS_PATH: &str = "/etc/quick-node/users.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub ss_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub traffic_limit_bytes: u64,
    pub traffic_used_bytes: u64,
    pub enabled: bool,
    pub sub_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsersState {
    pub users: Vec<User>,
}

impl UsersState {
    pub fn load() -> Result<Self> {
        let data = std::fs::read_to_string(USERS_PATH)
            .context("Failed to read users.json. Run 'quick-node init' first.")?;
        serde_json::from_str(&data).context("Invalid users.json")
    }

    pub fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(USERS_PATH, data)?;
        Ok(())
    }

    pub fn init() -> Result<()> {
        let state = Self::default();
        state.save()
    }

    pub fn add(
        &mut self,
        name: &str,
        expires: Option<DateTime<Utc>>,
        traffic_limit: u64,
    ) -> Result<&User> {
        if self.users.iter().any(|u| u.name == name) {
            bail!("User '{}' already exists", name);
        }

        let user = User {
            name: name.to_string(),
            ss_key: ss::generate_key(),
            created_at: Utc::now(),
            expires_at: expires,
            traffic_limit_bytes: traffic_limit,
            traffic_used_bytes: 0,
            enabled: true,
            sub_token: generate_token(),
        };

        self.users.push(user);
        Ok(self.users.last().unwrap())
    }

    pub fn remove(&mut self, name: &str) -> Result<User> {
        let idx = self
            .users
            .iter()
            .position(|u| u.name == name)
            .context(format!("User '{}' not found", name))?;
        Ok(self.users.remove(idx))
    }

    pub fn find(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|u| u.name == name)
    }
}

fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..16).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
}

pub fn parse_duration(s: &str) -> Result<chrono::Duration> {
    let s = s.trim();
    if s == "0" {
        bail!("no expiry");
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str.parse().context("Invalid duration number")?;

    match unit {
        "m" => Ok(chrono::Duration::minutes(num)),
        "h" => Ok(chrono::Duration::hours(num)),
        "d" => Ok(chrono::Duration::days(num)),
        "w" => Ok(chrono::Duration::weeks(num)),
        _ => bail!("Unknown duration unit '{}'. Use m/h/d/w", unit),
    }
}

pub fn parse_traffic_limit(s: &str) -> Result<u64> {
    let s = s.trim();
    if s == "0" {
        return Ok(0);
    }

    let s_upper = s.to_uppercase();
    if let Some(num) = s_upper.strip_suffix("TB") {
        return Ok(num.trim().parse::<u64>()? * 1024 * 1024 * 1024 * 1024);
    }
    if let Some(num) = s_upper.strip_suffix("GB") {
        return Ok(num.trim().parse::<u64>()? * 1024 * 1024 * 1024);
    }
    if let Some(num) = s_upper.strip_suffix("MB") {
        return Ok(num.trim().parse::<u64>()? * 1024 * 1024);
    }

    bail!("Unknown traffic format '{}'. Use MB/GB/TB", s);
}
