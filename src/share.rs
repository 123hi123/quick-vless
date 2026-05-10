use crate::config::AppConfig;
use crate::user::User;

pub fn hy2_url(config: &AppConfig, user: &User) -> String {
    format!(
        "hysteria2://{}@{}:{}/?insecure=1#{}",
        user.password, config.server_ip, config.hy_port, user.name
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

    serde_yaml::to_string(&serde_yaml::Value::Mapping({
        let mut root = serde_yaml::Mapping::new();

        let mut proxy = serde_yaml::Mapping::new();
        proxy.insert(y("name"), y(&proxy_name));
        proxy.insert(y("type"), y("hysteria2"));
        proxy.insert(y("server"), y(&config.server_ip));
        proxy.insert(
            y("port"),
            serde_yaml::Value::Number(config.hy_port.into()),
        );
        proxy.insert(y("password"), y(&user.password));
        proxy.insert(y("skip-cert-verify"), serde_yaml::Value::Bool(true));
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

pub fn socks5_url(config: &AppConfig) -> String {
    format!(
        "socks5://proxy:{}@{}:{}",
        config.socks_pass, config.server_ip, config.socks_port
    )
}

pub fn print_links(config: &AppConfig, user: &User) {
    use colored::Colorize;

    println!();
    println!("{}", format!("=== {} ===", user.name).bold());
    println!();
    println!("{}", "Hysteria2:".cyan().bold());
    println!("  {}", hy2_url(config, user));
    println!();
    println!("{}", "SOCKS5:".yellow().bold());
    println!("  {}", socks5_url(config));
    println!();
    println!("{}", "Clash Subscribe:".green().bold());
    println!("  {}", clash_sub_url(config, user));
    println!();
}
