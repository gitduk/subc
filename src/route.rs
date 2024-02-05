use std::vec;

use subc::{generate_proxies, generate_proxy_groups, generate_rules, get_nodes};

pub async fn _env() -> serde_json::Value {
    let env_vars = std::env::vars().collect::<std::collections::HashMap<String, String>>();
    serde_json::json!(env_vars)
}

pub async fn sub() -> String {

    // check config exists
    let config_arr = vec!["clash/config.yaml", "clash/groups.toml", "clash/rulesets.toml"];
    for config in config_arr {
        if !std::path::Path::new(config).exists() {
            return format!("{config} is not found.").to_string();
        }
    }

    // read clash basic config
    let config: String = match std::fs::read_to_string("clash/config.yaml") {
        Ok(c) => c,
        Err(e) => {
            return format!("read config.yaml failed: {e}");
        },
    }; 

    // load dotfile
    if let Err(e) = dotenvy::from_path(std::path::Path::new("clash/.env")) {
        return format!("load .env failed: {e}");
    }

    // generate proxies
    let url = match std::env::var("URL") {
        Ok(url) => url,
        Err(_) => {
            return "URL is not set".to_string();
        }
    };

    let res = match get_nodes(url).await {
        Ok(res) => res,
        Err(e) => {
            return format!("Get nodes failed: {e}");
        }
    };

    let (proxies, nodes) = generate_proxies(res);

    // generate proxy-groups
    let proxy_groups = match generate_proxy_groups("clash/groups.toml", nodes) {
        Ok(g) => g,
        Err(e) => {
            return format!("Generate proxy-groups failed: {e}");
        }
    };

    // generate rules
    let rules = match generate_rules("clash/rulesets.toml") {
        Ok(r) => r,
        Err(e) => {
            return format!("Generate rules failed: {e}");
        }
    };

    config.trim().to_string() + "\n\n" + &proxies + "\n" + &proxy_groups + "\n" + &rules
}
