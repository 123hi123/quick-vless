use crate::config::AppConfig;
use crate::user::User;

pub fn vless_url(config: &AppConfig, user: &User) -> String {
    format!(
        "vless://{}@{}:{}?encryption=none&flow=xtls-rprx-vision&security=reality&sni={}&fp=chrome&pbk={}&sid={}&type=tcp#{}",
        user.uuid,
        config.server_ip,
        config.vless_port,
        config.server_name,
        config.public_key,
        config.short_id,
        user.name,
    )
}

pub fn socks5_url(config: &AppConfig, user: &User) -> String {
    format!(
        "socks5://{}:{}@{}:{}",
        user.name,
        user.socks_pass,
        config.server_ip,
        config.socks_port,
    )
}

pub fn clash_sub_url(config: &AppConfig, user: &User) -> String {
    format!(
        "http://{}:{}/sub/{}",
        config.server_ip,
        config.sub_port,
        user.sub_token,
    )
}

pub fn clash_yaml(config: &AppConfig, user: &User) -> String {
    let proxy_name = format!("qv-{}", user.name);

    serde_yaml::to_string(&serde_yaml::Value::Mapping({
        let mut root = serde_yaml::Mapping::new();

        // proxies
        let mut proxy = serde_yaml::Mapping::new();
        proxy.insert(y("name"), y(&proxy_name));
        proxy.insert(y("type"), y("vless"));
        proxy.insert(y("server"), y(&config.server_ip));
        proxy.insert(y("port"), serde_yaml::Value::Number(config.vless_port.into()));
        proxy.insert(y("uuid"), y(&user.uuid.to_string()));
        proxy.insert(y("network"), y("tcp"));
        proxy.insert(y("tls"), serde_yaml::Value::Bool(true));
        proxy.insert(y("udp"), serde_yaml::Value::Bool(true));
        proxy.insert(y("flow"), y("xtls-rprx-vision"));
        proxy.insert(y("servername"), y(&config.server_name));
        proxy.insert(y("client-fingerprint"), y("chrome"));

        let mut reality_opts = serde_yaml::Mapping::new();
        reality_opts.insert(y("public-key"), y(&config.public_key));
        reality_opts.insert(y("short-id"), y(&config.short_id));
        proxy.insert(y("reality-opts"), serde_yaml::Value::Mapping(reality_opts));

        root.insert(
            y("proxies"),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::Mapping(proxy)]),
        );

        // proxy-groups
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

        // rules
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
    println!("{}", "VLESS:".cyan().bold());
    println!("  {}", vless_url(config, user));
    println!();
    println!("{}", "Clash Subscribe:".green().bold());
    println!("  {}", clash_sub_url(config, user));
    println!();
    println!("{}", "SOCKS5:".yellow().bold());
    println!("  {}", socks5_url(config, user));
    println!();
}
