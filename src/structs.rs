use std::vec;

use serde::{Deserialize, Serialize};

// basic config
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "mixed-port")]
    pub mixed_port: u32,
    #[serde(rename = "redir-port")]
    pub redir_port: u32,
    #[serde(rename = "allow-lan", default)]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level", default)]
    pub log_level: String,
    pub secret: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mixed_port: 7890,
            redir_port: 7891,
            allow_lan: true,
            mode: String::from("Rule"),
            log_level: String::from("debug"),
            secret: String::from("clash"),
            external_controller: String::from(":9090"),
        }
    }
}

// proxy-groups
#[derive(Debug, Deserialize)]
pub struct Groups {
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub interval: u32,
    #[serde(default)]
    pub tolerance: u32,
    #[serde(default)]
    pub strategy: String,
    pub proxies: Vec<String>,
}

impl Default for Group {
    fn default() -> Self {
        Group {
            name: String::from(""),
            group_type: String::from("select"),
            url: String::from("http://www.gstatic.com/generate_204"),
            interval: 180,
            tolerance: 180,
            strategy: String::from("round-robin"),
            proxies: vec![],
        }
    }
}

// rules
#[derive(Debug, Serialize, Deserialize)]
pub struct Rulesets {
    pub rulesets: Vec<Ruleset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ruleset {
    pub group: String,
    pub ruleset: String,
}

impl Default for Ruleset {
    fn default() -> Self {
        Ruleset {
            group: String::from(""),
            ruleset: String::from(""),
        }
    }
}
