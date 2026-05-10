use anyhow::{bail, Context, Result};
use rand::RngCore;
use std::process::Command;

pub fn generate_password() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
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

pub fn generate_tls_cert() -> Result<()> {
    println!("Generating TLS certificate...");
    let status = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-nodes",
            "-newkey",
            "ec",
            "-pkeyopt",
            "ec_paramgen_curve:prime256v1",
            "-days",
            "3650",
            "-subj",
            "/CN=bing.com",
            "-keyout",
            "/etc/quick-node/key.pem",
            "-out",
            "/etc/quick-node/cert.pem",
        ])
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to run openssl")?;

    if !status.success() {
        bail!("TLS certificate generation failed");
    }
    Ok(())
}

pub fn download_singbox() -> Result<()> {
    println!("Fetching latest sing-box release...");

    let output = Command::new("curl")
        .args([
            "-sL",
            "-H",
            "Accept: application/json",
            "https://api.github.com/repos/SagerNet/sing-box/releases/latest",
        ])
        .output()
        .context("Failed to query GitHub API")?;

    let body = String::from_utf8(output.stdout)?;
    let release: serde_json::Value =
        serde_json::from_str(&body).context("Failed to parse GitHub API response")?;
    let tag = release["tag_name"]
        .as_str()
        .context("Could not find tag_name in release")?;

    let version = tag.strip_prefix('v').unwrap_or(tag);

    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => bail!("Unsupported architecture: {}", std::env::consts::ARCH),
    };

    let filename = format!("sing-box-{}-linux-{}.tar.gz", version, arch);

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

    let tmp_tar = "/tmp/sing-box.tar.gz";
    let tmp_dir = "/tmp/sing-box-extract";

    println!("Downloading {}...", filename);
    let status = Command::new("curl")
        .args(["-sL", "-o", tmp_tar, &asset_url])
        .status()
        .context("Failed to download sing-box")?;

    if !status.success() {
        bail!("Download failed");
    }

    std::fs::create_dir_all(tmp_dir)?;

    let status = Command::new("tar")
        .args(["-xzf", tmp_tar, "-C", tmp_dir])
        .status()
        .context("Failed to extract archive")?;

    if !status.success() {
        bail!("Extraction failed");
    }

    let bin_path = format!("{}/sing-box-{}-linux-{}/sing-box", tmp_dir, version, arch);
    std::fs::copy(&bin_path, "/usr/local/bin/sing-box")?;
    Command::new("chmod")
        .args(["+x", "/usr/local/bin/sing-box"])
        .status()?;

    let _ = std::fs::remove_file(tmp_tar);
    let _ = std::fs::remove_dir_all(tmp_dir);

    println!("sing-box {} installed to /usr/local/bin/", tag);
    Ok(())
}

pub fn restart() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["restart", "sing-box"])
        .status()
        .context("Failed to restart sing-box")?;

    if !status.success() {
        bail!("systemctl restart sing-box failed");
    }
    Ok(())
}

pub fn status() -> Result<bool> {
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", "sing-box"])
        .status();

    Ok(status.map(|s| s.success()).unwrap_or(false))
}

pub fn install_systemd_units() -> Result<()> {
    let singbox_service = r#"[Unit]
Description=sing-box Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/sing-box run -c /etc/quick-node/singbox-config.json
Restart=on-failure
RestartSec=3
LimitNOFILE=65535

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

    std::fs::write("/etc/systemd/system/sing-box.service", singbox_service)?;
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

    for unit in &["sing-box", "quick-node-serve", "quick-node-check.timer"] {
        Command::new("systemctl")
            .args(["enable", "--now", unit])
            .status()?;
    }

    Ok(())
}
