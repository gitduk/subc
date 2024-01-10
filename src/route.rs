use subc::{generate_proxies, generate_proxy_groups, generate_rules, get_nodes, structs::*};

pub async fn _env() -> serde_json::Value {
    let env_vars = std::env::vars().collect::<std::collections::HashMap<String, String>>();
    serde_json::json!(env_vars)
}

pub async fn sub() -> String {
    // generate clash basic config
    let config: Config = toml::from_str(
        &std::fs::read_to_string("clash/config.toml").expect("read config.toml error."),
    )
    .expect("parse config.toml error.");
    let config = serde_yaml::to_string(&config).unwrap();

    // generate proxies
    let res = match std::env::var("URL") {
        Ok(url) => get_nodes(url).await,
        Err(_) => vec![],
    };
    let (proxyes, nodes) = generate_proxies(res);

    // generate proxy-groups
    let proxy_groups = generate_proxy_groups("clash/groups.toml", nodes);

    // generate rules
    let rules = generate_rules("clash/rulesets.toml");

    config + "\n" + &proxyes + "\n" + &proxy_groups + "\n" + &rules
}
