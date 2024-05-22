use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProxyGroup {
    Select {
        name: String,
        proxies: Vec<String>,
    },
    #[serde(rename = "load-balance")]
    LoadBalance {
        name: String,
        strategy: String,
        url: String,
        interval: u64,
        proxies: Vec<String>,
    },
    #[serde(rename = "url-test")]
    UrlTest {
        name: String,
        url: String,
        interval: u64,
        tolerance: u64,
        proxies: Vec<String>,
    },
    FallBack {
        name: String,
        url: String,
        interval: u64,
        tolerance: u64,
        proxies: Vec<String>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(rename = "mixed-port")]
    pub mixed_port: u64,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    #[serde(rename = "bind-address")]
    pub bind_address: String,
    pub mode: String,
    #[serde(default)]
    pub proxies: Vec<serde_json::Value>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mixed_port: 7890,
            allow_lan: true,
            bind_address: String::from("*"),
            mode: String::from("rule"),
            proxies: Vec::new(),
            proxy_groups: Vec::new(),
            rules: Vec::new(),
        }
    }
}
