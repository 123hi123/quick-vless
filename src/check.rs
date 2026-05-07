use anyhow::Result;
use chrono::Utc;
use colored::Colorize;

use crate::config::AppConfig;
use crate::user::UsersState;
use crate::xray;

pub fn run_check() -> Result<()> {
    let config = AppConfig::load()?;
    let mut state = UsersState::load()?;

    let stats = xray::query_stats(&config.xray_api_addr).unwrap_or_default();

    for traffic in &stats {
        if let Some(user) = state.users.iter_mut().find(|u| u.email == traffic.email) {
            user.traffic_used_bytes += traffic.uplink + traffic.downlink;
        }
    }

    let now = Utc::now();
    let mut needs_reload = false;

    for user in state.users.iter_mut().filter(|u| u.enabled) {
        let mut reason = None;

        if let Some(expires) = user.expires_at {
            if now >= expires {
                reason = Some("expired");
            }
        }

        if user.traffic_limit_bytes > 0 && user.traffic_used_bytes >= user.traffic_limit_bytes {
            reason = Some("traffic limit exceeded");
        }

        if let Some(r) = reason {
            println!(
                "{} User '{}' disabled: {}",
                "[CHECK]".red().bold(),
                user.name,
                r
            );
            user.enabled = false;
            needs_reload = true;
        }
    }

    state.save()?;

    if needs_reload {
        config.generate_xray_config(&state.users)?;
        xray::restart_xray()?;
        println!("{} Xray config reloaded", "[CHECK]".green().bold());
    }

    Ok(())
}
