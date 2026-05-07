mod check;
mod config;
mod serve;
mod share;
mod user;
mod xray;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use config::AppConfig;
use user::UsersState;

#[derive(Parser)]
#[command(name = "quick-vless", version, about = "Manage VLESS + Reality proxy nodes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize: download Xray-core, generate Reality keys, setup systemd
    Init {
        /// VLESS listen port
        #[arg(short, long, default_value = "443")]
        port: u16,

        /// SNI target for Reality camouflage
        #[arg(short, long, default_value = "www.microsoft.com")]
        sni: String,

        /// SOCKS5 listen port
        #[arg(long, default_value = "1080")]
        socks_port: u16,

        /// HTTP subscription server port
        #[arg(long, default_value = "8443")]
        sub_port: u16,

        /// Server IP (auto-detected if omitted)
        #[arg(long)]
        ip: Option<String>,
    },

    /// Manage users
    User {
        #[command(subcommand)]
        command: UserCommands,
    },

    /// Re-detect public IP and update all links
    Refresh,

    /// Run periodic check (traffic/expiry), called by systemd timer
    Check,

    /// Show server status
    Status,

    /// Start HTTP subscription server
    Serve,
}

#[derive(Subcommand)]
enum UserCommands {
    /// Add a new user
    Add {
        /// User name
        name: String,

        /// Expiry duration (e.g. 30d, 6h, 1w, 0=never)
        #[arg(short, long, default_value = "30d")]
        expires: String,

        /// Traffic limit (e.g. 100GB, 500MB, 0=unlimited)
        #[arg(short, long, default_value = "0")]
        traffic_limit: String,
    },
    /// List all users
    List,
    /// Remove a user
    Remove {
        /// User name
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            port,
            sni,
            socks_port,
            sub_port,
            ip,
        } => cmd_init(port, sni, socks_port, sub_port, ip)?,

        Commands::User { command } => match command {
            UserCommands::Add {
                name,
                expires,
                traffic_limit,
            } => cmd_user_add(&name, &expires, &traffic_limit)?,
            UserCommands::List => cmd_user_list()?,
            UserCommands::Remove { name } => cmd_user_remove(&name)?,
        },

        Commands::Refresh => cmd_refresh()?,
        Commands::Check => check::run_check()?,
        Commands::Status => cmd_status()?,
        Commands::Serve => {
            let config = AppConfig::load()?;
            serve::run_server(&config).await?;
        }
    }

    Ok(())
}

fn cmd_init(
    port: u16,
    sni: String,
    socks_port: u16,
    sub_port: u16,
    ip: Option<String>,
) -> Result<()> {
    println!("{}", "=== Quick-VLESS Init ===".bold());

    let server_ip = match ip {
        Some(ip) => ip,
        None => {
            println!("Detecting public IP...");
            xray::detect_public_ip()?
        }
    };
    println!("Server IP: {}", server_ip.green());

    xray::download_xray()?;

    println!("Generating Reality keypair...");
    let (private_key, public_key) = xray::generate_keypair();
    let short_id = xray::generate_short_id();

    let config = AppConfig {
        server_ip,
        vless_port: port,
        private_key,
        public_key: public_key.clone(),
        short_id: short_id.clone(),
        server_name: sni.clone(),
        xray_api_addr: "127.0.0.1:10085".to_string(),
        sub_port,
        socks_port,
    };

    config.save()?;
    UsersState::init()?;
    config.generate_xray_config(&[])?;

    println!("Installing systemd units...");
    xray::install_systemd_units()?;

    println!();
    println!("{}", "=== Init Complete ===".green().bold());
    println!("  VLESS port:  {}", port);
    println!("  SOCKS5 port: {}", socks_port);
    println!("  Sub port:    {}", sub_port);
    println!("  SNI:         {}", sni);
    println!("  Public Key:  {}", public_key);
    println!("  Short ID:    {}", short_id);
    println!();
    println!(
        "Next: {} to create a user and get share links",
        "quick-vless user add <name>".cyan()
    );

    Ok(())
}

fn cmd_user_add(name: &str, expires_str: &str, traffic_str: &str) -> Result<()> {
    let config = AppConfig::load()?;
    let mut state = UsersState::load()?;

    let expires_at = match user::parse_duration(expires_str) {
        Ok(d) => Some(chrono::Utc::now() + d),
        Err(_) => None,
    };
    let traffic_limit = user::parse_traffic_limit(traffic_str)?;

    let _user = state.add(name, expires_at, traffic_limit)?;
    state.save()?;

    config.generate_xray_config(&state.users)?;

    let user = state.find(name).unwrap();
    share::save_clash_sub(&config, user)?;

    xray::restart_xray()?;

    share::print_links(&config, user);

    if let Some(exp) = &user.expires_at {
        println!(
            "  Expires: {}",
            exp.format("%Y-%m-%d %H:%M UTC").to_string().yellow()
        );
    }
    if user.traffic_limit_bytes > 0 {
        println!(
            "  Traffic limit: {}",
            format_bytes(user.traffic_limit_bytes).yellow()
        );
    }

    Ok(())
}

fn cmd_user_list() -> Result<()> {
    let config = AppConfig::load()?;
    let state = UsersState::load()?;

    if state.users.is_empty() {
        println!("No users.");
        return Ok(());
    }

    for user in &state.users {
        let status = if user.enabled {
            "ACTIVE".green()
        } else {
            "DISABLED".red()
        };

        println!("{} [{}]", user.name.bold(), status);
        println!("  UUID:    {}", user.uuid);
        println!("  Traffic: {} / {}",
            format_bytes(user.traffic_used_bytes),
            if user.traffic_limit_bytes > 0 {
                format_bytes(user.traffic_limit_bytes)
            } else {
                "unlimited".to_string()
            }
        );

        if let Some(exp) = &user.expires_at {
            let remaining = *exp - chrono::Utc::now();
            let remaining_str = if remaining.num_seconds() < 0 {
                "EXPIRED".red().to_string()
            } else if remaining.num_days() > 0 {
                format!("{}d left", remaining.num_days())
            } else {
                format!("{}h left", remaining.num_hours())
            };
            println!("  Expires: {} ({})", exp.format("%Y-%m-%d %H:%M UTC"), remaining_str);
        } else {
            println!("  Expires: never");
        }

        share::print_links(&config, user);
    }

    Ok(())
}

fn cmd_user_remove(name: &str) -> Result<()> {
    let config = AppConfig::load()?;
    let mut state = UsersState::load()?;

    let removed = state.remove(name)?;
    state.save()?;

    config.generate_xray_config(&state.users)?;

    // remove subscription file
    let sub_path = format!("{}/subs/{}.yaml", AppConfig::config_dir(), removed.sub_token);
    let _ = std::fs::remove_file(sub_path);

    xray::restart_xray()?;

    println!("User '{}' removed.", name.yellow());
    Ok(())
}

fn cmd_refresh() -> Result<()> {
    let mut config = AppConfig::load()?;
    let state = UsersState::load()?;

    println!("Detecting public IP...");
    let new_ip = xray::detect_public_ip()?;

    if new_ip == config.server_ip {
        println!("IP unchanged: {}", new_ip.green());
        println!("No update needed.");
    } else {
        println!(
            "IP changed: {} → {}",
            config.server_ip.red(),
            new_ip.green()
        );
        config.server_ip = new_ip;
        config.save()?;
    }

    share::save_all_clash_subs(&config, &state.users)?;

    for user in &state.users {
        share::print_links(&config, user);
    }

    Ok(())
}

fn cmd_status() -> Result<()> {
    let config = AppConfig::load()?;
    let state = UsersState::load()?;

    let xray_running = xray::xray_status()?;

    println!("{}", "=== Quick-VLESS Status ===".bold());
    println!();
    println!(
        "  Xray:     {}",
        if xray_running {
            "running".green()
        } else {
            "stopped".red()
        }
    );
    println!("  Server:   {}", config.server_ip);
    println!("  VLESS:    port {}", config.vless_port);
    println!("  SOCKS5:   port {}", config.socks_port);
    println!("  Sub HTTP: port {}", config.sub_port);
    println!("  SNI:      {}", config.server_name);
    println!("  PubKey:   {}", config.public_key);
    println!("  ShortID:  {}", config.short_id);
    println!();

    let total = state.users.len();
    let active = state.users.iter().filter(|u| u.enabled).count();
    println!("  Users: {} total, {} active", total, active);

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} B", bytes)
    }
}
