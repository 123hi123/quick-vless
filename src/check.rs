use anyhow::Result;
use chrono::Utc;
use colored::Colorize;

use crate::config::AppConfig;
use crate::ss;
use crate::user::UsersState;

pub fn run_check() -> Result<()> {
    let config = AppConfig::load()?;
    let mut state = UsersState::load()?;

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
        config.generate_ss_config(&state.users)?;
        ss::restart_ss()?;
        println!("{} ssserver config reloaded", "[CHECK]".green().bold());
    }

    Ok(())
}
