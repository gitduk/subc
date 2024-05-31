use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub nodes: Vec<serde_json::Value>,
}

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
pub struct ProxyProvider {
    pub proxies: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub proxies: Vec<serde_json::Value>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            proxies: Vec::new(),
            proxy_groups: Vec::new(),
            rules: Vec::new(),
        }
    }
}
