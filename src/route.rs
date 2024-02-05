use subc::{generate_proxies, generate_proxy_groups, generate_rules, get_nodes};

pub async fn _env() -> serde_json::Value {
    let env_vars = std::env::vars().collect::<std::collections::HashMap<String, String>>();
    serde_json::json!(env_vars)
}

pub async fn sub() -> String {
    // read clash basic config
    let config: String = std::fs::read_to_string("clash/conf/config.yaml").expect("read config.toml error.");

    // generate proxies
    let res = match std::env::var("URL") {
        Ok(url) => get_nodes(url).await,
        Err(_) => vec![],
    };
    let (proxyes, nodes) = generate_proxies(res);

    // generate proxy-groups
    let proxy_groups = generate_proxy_groups("clash/conf/groups.toml", nodes);

    // generate rules
    let rules = generate_rules("clash/conf/rulesets.toml");

    config.trim().to_string() + "\n\n" + &proxyes + "\n" + &proxy_groups + "\n" + &rules
}
