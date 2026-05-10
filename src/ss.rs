use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
use std::process::Command;

pub fn generate_key() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    STANDARD.encode(key)
}

pub fn detect_public_ip() -> Result<String> {
    let output = Command::new("curl")
        .args(["-s", "-4", "--max-time", "10", "https://ifconfig.me"])
        .output()
        .context("Failed to run curl")?;

    let ip = String::from_utf8(output.stdout)?.trim().to_string();
    if ip.is_empty() {
        bail!("Could not detect public IP");
    }
    Ok(ip)
}

pub fn download_ssserver() -> Result<()> {
    println!("Fetching latest shadowsocks-rust release...");

    let output = Command::new("curl")
        .args([
            "-sL",
            "-H",
            "Accept: application/json",
            "https://api.github.com/repos/shadowsocks/shadowsocks-rust/releases/latest",
        ])
        .output()
        .context("Failed to query GitHub API")?;

    let body = String::from_utf8(output.stdout)?;
    let release: serde_json::Value =
        serde_json::from_str(&body).context("Failed to parse GitHub API response")?;
    let tag = release["tag_name"]
        .as_str()
        .context("Could not find tag_name in release")?;

    let arch = std::env::consts::ARCH;
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let filename = format!("shadowsocks-{}.{}.tar.xz", tag, target);

    let assets = release["assets"]
        .as_array()
        .context("No assets in release")?;
    let asset_url = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n == filename)
                .unwrap_or(false)
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .context(format!("Asset '{}' not found in release", filename))?
        .to_string();

    let tmp_tar = "/tmp/shadowsocks.tar.xz";
    let tmp_dir = "/tmp/shadowsocks-extract";

    println!("Downloading {}...", filename);
    let status = Command::new("curl")
        .args(["-sL", "-o", tmp_tar, &asset_url])
        .status()
        .context("Failed to download shadowsocks-rust")?;

    if !status.success() {
        bail!("Download failed");
    }

    std::fs::create_dir_all(tmp_dir)?;

    let status = Command::new("tar")
        .args(["-xf", tmp_tar, "-C", tmp_dir])
        .status()
        .context("Failed to extract archive (is 'xz' installed?)")?;

    if !status.success() {
        bail!("Extraction failed");
    }

    std::fs::copy(format!("{}/ssserver", tmp_dir), "/usr/local/bin/ssserver")?;
    std::fs::copy(format!("{}/sslocal", tmp_dir), "/usr/local/bin/sslocal")?;
    Command::new("chmod")
        .args(["+x", "/usr/local/bin/ssserver"])
        .status()?;
    Command::new("chmod")
        .args(["+x", "/usr/local/bin/sslocal"])
        .status()?;

    let _ = std::fs::remove_file(tmp_tar);
    let _ = std::fs::remove_dir_all(tmp_dir);

    println!("ssserver + sslocal installed to /usr/local/bin/");
    Ok(())
}

pub fn restart_ss() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["restart", "ssserver"])
        .status()
        .context("Failed to restart ssserver")?;

    if !status.success() {
        bail!("systemctl restart ssserver failed");
    }
    Ok(())
}

pub fn ss_status() -> Result<bool> {
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", "ssserver"])
        .status();

    Ok(status.map(|s| s.success()).unwrap_or(false))
}

pub fn sslocal_status() -> Result<bool> {
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", "sslocal"])
        .status();

    Ok(status.map(|s| s.success()).unwrap_or(false))
}

pub fn install_systemd_units() -> Result<()> {
    let ss_service = r#"[Unit]
Description=Shadowsocks Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/ssserver -c /etc/quick-node/ss-config.json
Restart=on-failure
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
"#;

    let sslocal_service = r#"[Unit]
Description=Shadowsocks Local (SOCKS5 Bridge)
After=ssserver.service
Requires=ssserver.service

[Service]
Type=simple
ExecStart=/usr/local/bin/sslocal -c /etc/quick-node/sslocal-config.json
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
"#;

    let serve_service = r#"[Unit]
Description=Quick-Node Subscription Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/quick-node serve
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
"#;

    let check_service = r#"[Unit]
Description=Quick-Node Check

[Service]
Type=oneshot
ExecStart=/usr/local/bin/quick-node check
"#;

    let check_timer = r#"[Unit]
Description=Quick-Node periodic check

[Timer]
OnBootSec=5min
OnUnitActiveSec=10min
Persistent=true

[Install]
WantedBy=timers.target
"#;

    std::fs::write("/etc/systemd/system/ssserver.service", ss_service)?;
    std::fs::write("/etc/systemd/system/sslocal.service", sslocal_service)?;
    std::fs::write(
        "/etc/systemd/system/quick-node-serve.service",
        serve_service,
    )?;
    std::fs::write(
        "/etc/systemd/system/quick-node-check.service",
        check_service,
    )?;
    std::fs::write(
        "/etc/systemd/system/quick-node-check.timer",
        check_timer,
    )?;

    Command::new("systemctl").arg("daemon-reload").status()?;

    for unit in &[
        "ssserver",
        "sslocal",
        "quick-node-serve",
        "quick-node-check.timer",
    ] {
        Command::new("systemctl")
            .args(["enable", "--now", unit])
            .status()?;
    }

    Ok(())
}
