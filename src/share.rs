use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::config::AppConfig;
use crate::user::User;

pub fn ss_url(config: &AppConfig, user: &User) -> String {
    let userinfo = format!(
        "2022-blake3-aes-256-gcm:{}:{}",
        config.server_key, user.ss_key
    );
    let encoded = URL_SAFE_NO_PAD.encode(userinfo.as_bytes());
    format!(
        "ss://{}@{}:{}#{}",
        encoded, config.server_ip, config.ss_port, user.name
    )
}

pub fn clash_sub_url(config: &AppConfig, user: &User) -> String {
    format!(
        "http://{}:{}/sub/{}",
        config.server_ip, config.sub_port, user.sub_token,
    )
}

pub fn clash_yaml(config: &AppConfig, user: &User) -> String {
    let proxy_name = format!("qn-{}", user.name);
    let password = format!("{}:{}", config.server_key, user.ss_key);

    serde_yaml::to_string(&serde_yaml::Value::Mapping({
        let mut root = serde_yaml::Mapping::new();

        let mut proxy = serde_yaml::Mapping::new();
        proxy.insert(y("name"), y(&proxy_name));
        proxy.insert(y("type"), y("ss"));
        proxy.insert(y("server"), y(&config.server_ip));
        proxy.insert(
            y("port"),
            serde_yaml::Value::Number(config.ss_port.into()),
        );
        proxy.insert(y("cipher"), y("2022-blake3-aes-256-gcm"));
        proxy.insert(y("password"), y(&password));
        proxy.insert(y("udp"), serde_yaml::Value::Bool(true));

        root.insert(
            y("proxies"),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::Mapping(proxy)]),
        );

        let mut group = serde_yaml::Mapping::new();
        group.insert(y("name"), y("PROXY"));
        group.insert(y("type"), y("select"));
        group.insert(
            y("proxies"),
            serde_yaml::Value::Sequence(vec![y(&proxy_name)]),
        );
        root.insert(
            y("proxy-groups"),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::Mapping(group)]),
        );

        root.insert(
            y("rules"),
            serde_yaml::Value::Sequence(vec![y("MATCH,PROXY")]),
        );

        root
    }))
    .unwrap_or_default()
}

fn y(s: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(s.to_string())
}

pub fn save_clash_sub(config: &AppConfig, user: &User) -> anyhow::Result<()> {
    let dir = format!("{}/subs", AppConfig::config_dir());
    std::fs::create_dir_all(&dir)?;
    let path = format!("{}/{}.yaml", dir, user.sub_token);
    std::fs::write(path, clash_yaml(config, user))?;
    Ok(())
}

pub fn save_all_clash_subs(config: &AppConfig, users: &[User]) -> anyhow::Result<()> {
    for user in users.iter().filter(|u| u.enabled) {
        save_clash_sub(config, user)?;
    }
    Ok(())
}

pub fn print_links(config: &AppConfig, user: &User) {
    use colored::Colorize;

    println!();
    println!("{}", format!("=== {} ===", user.name).bold());
    println!();
    println!("{}", "SS:".cyan().bold());
    println!("  {}", ss_url(config, user));
    println!();
    println!("{}", "Clash Subscribe:".green().bold());
    println!("  {}", clash_sub_url(config, user));
    println!();
}
