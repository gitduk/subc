use anyhow::{Error, Result};
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
};
use regex::Regex;
use reqwest::Proxy;
use std::{collections::HashMap, path::StripPrefixError};
use tokio::io::AsyncBufReadExt;

use crate::structs::{Config, ProxyGroup};
use crate::utils::{base64_decode, base64_decode_no_pad};

const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const AC: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7";

pub const DEFAULT_CONFIG: &str = r#"
mixed-port: 7890
allow-lan: true
bind-address: "*"
mode: rule

proxy-groups:
  - name: "🌻 节点选择"
    type: "select"
    proxies:
      - "[]🌲 负载均衡-轮询"
      - "[]🍁 负载均衡-散列"
      - "[]🐏 手动切换"

  - name: "🌲 负载均衡-轮询"
    type: "load-balance"
    strategy: "round-robin"
    url: "http://www.gstatic.com/generate_204"
    interval: 180
    proxies:
      - "香港[^-]|台湾|日本"

  - name: "🍁 负载均衡-散列"
    type: "load-balance"
    strategy: "consistent-hashing"
    url: "http://www.gstatic.com/generate_204"
    interval: 180
    proxies:
      - "香港[^-]|台湾|日本"

  - name: "🐏 手动切换"
    type: "select"
    proxies:
      - ".*?"

  - name: 🪁 Match
    type: select
    proxies:
      - "[]🌻 节点选择"
      - DIRECT

rules:
  - group: DIRECT
    ruleset: lan_cidr.txt

  - group: 🪁 Match
    ruleset: MATCH
"#;

pub async fn from_url(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let url = params.get("url").unwrap();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return format!("Invalid url: {url}");
    }

    let config_path = xdg::BaseDirectories::with_prefix("clash")
        .unwrap()
        .place_config_file("subc.yaml")
        .unwrap();
    if !config_path.exists() {
        return format!("subc.yaml not exists");
    }
    let config_string = std::fs::read_to_string(config_path).unwrap();
    let mut config: Config = serde_yaml::from_str(&config_string).unwrap();

    // get nodes
    let nodes = get_nodes_from(url).await.unwrap();
    let nodes = nodes
        .iter()
        .filter(|node| {
            if let serde_json::Value::Object(map) = node {
                if map.is_empty() {
                    return false;
                }
            }
            true
        })
        .collect::<Vec<_>>();

    let proxy_name_vec: Vec<_> = nodes
        .iter()
        .map(|n| n.get("name").unwrap().to_string())
        .collect();

    config.proxies = nodes
        .iter()
        .map(|n| n.to_owned().clone())
        .collect::<Vec<_>>();

    // process proxy-groups
    config
        .proxy_groups
        .iter_mut()
        .for_each(|group| match group {
            ProxyGroup::Select { proxies, .. }
            | ProxyGroup::LoadBalance { proxies, .. }
            | ProxyGroup::UrlTest { proxies, .. }
            | ProxyGroup::FallBack { proxies, .. } => {
                let mut filtered_proxies: Vec<String> = vec![];
                for p in proxies.clone() {
                    if p.starts_with("[]") {
                        filtered_proxies.push(p.clone().replace("[]", ""));
                        continue;
                    }
                    proxy_name_vec.iter().for_each(|n| {
                        if regex::Regex::new(&p).unwrap().is_match(n) {
                            filtered_proxies.push(n.replace('"', ""));
                        }
                    });
                }
                *proxies = filtered_proxies;
            }
        });

    config.proxy_groups.retain(|group| match group {
        ProxyGroup::Select { proxies, .. }
        | ProxyGroup::LoadBalance { proxies, .. }
        | ProxyGroup::UrlTest { proxies, .. }
        | ProxyGroup::FallBack { proxies, .. } => !proxies.is_empty(),
    });

    // process rules
    let mut rules = vec![];
    config.rules.iter_mut().for_each(|line| {
        let mut parts = line.splitn(2, ",");
        let rule_path = parts.next().unwrap().to_string();
        let group = parts.next().unwrap_or("").to_string();
        let ruleset = xdg::BaseDirectories::with_prefix("clash/rules")
            .unwrap()
            .place_config_file(rule_path)
            .unwrap();
        if ruleset.exists() {
            let rule_string = std::fs::read_to_string(ruleset).unwrap();
            let rule_vec: Vec<_> = rule_string
                .lines()
                .filter(|r| !r.starts_with("#"))
                .map(|r| r.to_string() + "," + &group)
                .collect();
            rules.extend(rule_vec)
        } else {
            rules.push(line.clone())
        }
    });
    config.rules = rules;

    let config_value = serde_yaml::to_value(&config).unwrap();
    let config_string = serde_yaml::to_string(&config_value).unwrap();

    config_string
}

async fn get_nodes_from(url: &str) -> Result<Vec<serde_json::Value>, Error> {
    let res = reqwest::Client::new()
        .get(url)
        .header("User-Agent", UA)
        .header("Accept", AC)
        .send()
        .await?;
    let encoded = res.text().await?;
    let node_str = base64_decode(encoded);
    let nodes = node_str
        .lines()
        .into_iter()
        .map(|s| {
            let line = s.replacen("://", ":", 1).trim().to_string();
            Regex::new(r"[#:@]")
                .unwrap()
                .split(&line)
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .map(node_builder)
        .collect::<Vec<_>>();
    Ok(nodes)
}

fn node_builder(node: Vec<String>) -> serde_json::Value {
    let node_type = &node[0];
    let node = match node_type.as_str() {
        "ss" => {
            let cipher_password = &node[1];
            let server = &node[2];
            let port = &node[3];
            let name = &node[4];
            let name = urlencoding::decode(name).expect("UTF-8");
            let cipher_password = base64_decode_no_pad(cipher_password.to_owned());
            let cipher_password: Vec<&str> = cipher_password.split(":").collect();
            let cipher = cipher_password[0];
            let password = cipher_password[1];
            serde_json::json!({
                "name": name.to_string(),
                "server": server,
                "port": port,
                "type": *node_type,
                "cipher": cipher,
                "password": password
            })
        }
        // TODO: add other node type parser
        _ => serde_json::json!({}),
    };
    node
}
