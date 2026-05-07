use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use std::process::Command;
use x25519_dalek::{PublicKey, StaticSecret};

pub fn generate_keypair() -> (String, String) {
    let mut rng = rand::thread_rng();
    let secret = StaticSecret::random_from_rng(&mut rng);
    let public = PublicKey::from(&secret);

    let private_b64 = URL_SAFE_NO_PAD.encode(secret.as_bytes());
    let public_b64 = URL_SAFE_NO_PAD.encode(public.as_bytes());

    (private_b64, public_b64)
}

pub fn generate_short_id() -> String {
    let mut bytes = [0u8; 4];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(&bytes)
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

pub fn download_xray() -> Result<()> {
    let url = "https://github.com/XTLS/Xray-core/releases/latest/download/Xray-linux-64.zip";
    let tmp_zip = "/tmp/xray-core.zip";
    let tmp_dir = "/tmp/xray-core-extract";

    println!("Downloading Xray-core...");
    let status = Command::new("curl")
        .args(["-sL", "-o", tmp_zip, url])
        .status()
        .context("Failed to download Xray-core")?;

    if !status.success() {
        bail!("Download failed");
    }

    std::fs::create_dir_all(tmp_dir)?;

    let status = Command::new("unzip")
        .args(["-o", tmp_zip, "-d", tmp_dir])
        .status()
        .context("Failed to extract Xray-core (is 'unzip' installed?)")?;

    if !status.success() {
        bail!("Extraction failed");
    }

    std::fs::copy(format!("{}/xray", tmp_dir), "/usr/local/bin/xray")?;

    Command::new("chmod")
        .args(["+x", "/usr/local/bin/xray"])
        .status()?;

    // cleanup
    let _ = std::fs::remove_file(tmp_zip);
    let _ = std::fs::remove_dir_all(tmp_dir);

    println!("Xray-core installed to /usr/local/bin/xray");
    Ok(())
}

pub fn restart_xray() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["restart", "xray"])
        .status()
        .context("Failed to restart xray")?;

    if !status.success() {
        bail!("systemctl restart xray failed");
    }
    Ok(())
}

pub fn xray_status() -> Result<bool> {
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", "xray"])
        .status();

    Ok(status.map(|s| s.success()).unwrap_or(false))
}

pub fn install_systemd_units() -> Result<()> {
    let xray_service = r#"[Unit]
Description=Xray Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/xray run -config /etc/quick-vless/xray-config.json
Restart=on-failure
RestartSec=3
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
"#;

    let serve_service = r#"[Unit]
Description=Quick-VLESS Subscription Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/quick-vless serve
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
"#;

    let check_service = r#"[Unit]
Description=Quick-VLESS Check

[Service]
Type=oneshot
ExecStart=/usr/local/bin/quick-vless check
"#;

    let check_timer = r#"[Unit]
Description=Quick-VLESS periodic check

[Timer]
OnBootSec=5min
OnUnitActiveSec=10min
Persistent=true

[Install]
WantedBy=timers.target
"#;

    std::fs::write("/etc/systemd/system/xray.service", xray_service)?;
    std::fs::write(
        "/etc/systemd/system/quick-vless-serve.service",
        serve_service,
    )?;
    std::fs::write(
        "/etc/systemd/system/quick-vless-check.service",
        check_service,
    )?;
    std::fs::write(
        "/etc/systemd/system/quick-vless-check.timer",
        check_timer,
    )?;

    Command::new("systemctl").arg("daemon-reload").status()?;

    for unit in &[
        "xray",
        "quick-vless-serve",
        "quick-vless-check.timer",
    ] {
        Command::new("systemctl")
            .args(["enable", "--now", unit])
            .status()?;
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct StatsResponse {
    stat: Option<Vec<StatEntry>>,
}

#[derive(Debug, serde::Deserialize)]
struct StatEntry {
    name: String,
    value: Option<i64>,
}

pub struct UserTraffic {
    pub email: String,
    pub uplink: u64,
    pub downlink: u64,
}

pub fn query_stats(api_addr: &str) -> Result<Vec<UserTraffic>> {
    let output = Command::new("/usr/local/bin/xray")
        .args([
            "api",
            "statsquery",
            &format!("--server={}", api_addr),
            "-pattern",
            "user>>>",
            "-reset",
        ])
        .output()
        .context("Failed to query Xray stats")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Xray stats query failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    if stdout.trim().is_empty() {
        return Ok(vec![]);
    }

    let resp: StatsResponse =
        serde_json::from_str(&stdout).context("Failed to parse stats JSON")?;

    let entries = resp.stat.unwrap_or_default();
    let mut traffic_map: std::collections::HashMap<String, (u64, u64)> =
        std::collections::HashMap::new();

    for entry in entries {
        // format: "user>>>email@qv>>>traffic>>>uplink"
        let parts: Vec<&str> = entry.name.split(">>>").collect();
        if parts.len() == 4 {
            let email = parts[1].to_string();
            let value = entry.value.unwrap_or(0).max(0) as u64;
            let stat = traffic_map.entry(email).or_insert((0, 0));
            match parts[3] {
                "uplink" => stat.0 += value,
                "downlink" => stat.1 += value,
                _ => {}
            }
        }
    }

    Ok(traffic_map
        .into_iter()
        .map(|(email, (up, down))| UserTraffic {
            email,
            uplink: up,
            downlink: down,
        })
        .collect())
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
