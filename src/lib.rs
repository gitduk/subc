use regex::Regex;
use std::fs;
use std::path::Path;

pub mod decode;
pub mod structs;

use structs::*;

const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const AC: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7";

pub async fn env() -> serde_json::Value {
    let env_vars = std::env::vars().collect::<std::collections::HashMap<String, String>>();
    serde_json::json!(env_vars)
}

pub async fn get_nodes(url: String) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let res = reqwest::Client::new()
        .get(url)
        .header("User-Agent", UA)
        .header("Accept", AC)
        .send()
        .await?;
    let encoded = res.text().await?;
    let node_str = decode::base64_decode(encoded);

    let mut nodes = vec![];
    for n in node_str.lines() {
        let line = n
            .trim()
            .replace("//", "")
            .replace("@", ":")
            .replace("#", ":");
        let parts: Vec<&str> = line.split(':').collect();

        let type_ = parts[0];

        if type_ == "ss" {
            let (cipher_password, server, port, name) = (parts[1], parts[2], parts[3], parts[4]);
            let name = urlencoding::decode(name).expect("UTF-8");
            let cipher_password = decode::base64_decode_no_pad(cipher_password.to_owned());
            let cipher_password: Vec<&str> = cipher_password.split(":").collect();
            let cipher = cipher_password[0];
            let password = cipher_password[1];
            nodes.push(serde_json::json!({
                "name": name.to_string(),
                "server": server,
                "port": port,
                "type": type_,
                "cipher": cipher,
                "password": password
            }));
        }
    }
    Ok(nodes)
}

pub fn filter_node(pattern: &str, nodes: Vec<String>) -> Vec<String> {
    let mut filtered_node = vec![];
    for node_name in nodes {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => {
                println!("your pattern `{pattern}` is invalied. {e}");
                Regex::new("").unwrap()
            }
        };

        let is_match = re.is_match(&node_name);
        if is_match {
            filtered_node.push(node_name);
        }
    }
    filtered_node
}

pub fn generate_proxies(res: Vec<serde_json::Value>) -> (String, Vec<String>) {
    let mut nodes_string = String::from("proxies:\n");
    let mut nodes = vec![];
    for data in res {
        let name = data.get("name").unwrap().to_string();
        let cipher = data.get("cipher").unwrap().to_string();
        let password = data.get("password").unwrap().to_string();
        let server = data.get("server").unwrap().to_string();
        let port = data.get("port").unwrap().to_string();
        let node_type = data.get("type").unwrap().to_string();
        let node = format!(
            "  - {{name: {name}, server: {server}, port: {port}, type: {node_type}, cipher: {cipher}, password: {password}}}\n"
        );
        nodes_string.push_str(&node);
        nodes.push(name.replace("\"", ""));
    }
    (nodes_string, nodes)
}

pub fn generate_proxy_groups(path: &str, nodes: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    let group_string = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let error = match e.kind() {
                std::io::ErrorKind::NotFound => format!("{} is not found.", path),
                _ => format!("read file failed: {e}"),
            };
            return Err(error.into());
        }
    };

    let mut groups: Groups = match toml::from_str(&group_string) {
        Ok(g) => g,
        Err(e) => {
            return Err(e.into());
        }
    };

    let mut groups_str = String::from("proxy-groups:\n");
    for g in groups.proxy_groups.iter_mut() {
        // proxies
        let mut empty = false;
        let mut proxie_str = String::new();
        proxie_str.push_str("    proxies:\n");
        for r in g.proxies.iter() {
            if r.starts_with("[]") {
                proxie_str.push_str(&r.replace("[]", "      - "));
                proxie_str.push_str("\n");
            } else {
                let node_list = filter_node(r, nodes.clone());

                if node_list.is_empty() {
                    empty = true;
                    break;
                }
                for node in node_list {
                    proxie_str.push_str(&format!("      - {}\n", node));
                }
            }
        }

        if empty {
            continue;
        }

        // name
        groups_str.push_str(&format!("  - name: {}\n", g.name));

        // type
        groups_str.push_str(&format!("    type: {}\n", g.group_type));

        if g.group_type == "url-test" {
            groups_str.push_str(&format!("    url: {}\n", g.url));
            groups_str.push_str(&format!("    interval: {}\n", g.interval));
            groups_str.push_str(&format!("    tolerance: {}\n", g.tolerance));
        } else if g.group_type == "load-balance" {
            groups_str.push_str(&format!("    strategy: {}\n", g.strategy));
            groups_str.push_str(&format!("    url: {}\n", g.url));
            groups_str.push_str(&format!("    interval: {}\n", g.interval));
        }

        groups_str.push_str(&proxie_str);
    }

    Ok(groups_str)

}

pub fn generate_rules(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let rulesets_string = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            let error = match e.kind() {
                std::io::ErrorKind::NotFound => format!("{} is not found.", path),
                _ => format!("read file failed: {e}"),
            };
            return Err(error.into());
        }
    };

    let rulesets: Rulesets = match toml::from_str(&rulesets_string) {
        Ok(r) => r,
        Err(e) => {
            return Err(e.into());
        }
    };

    let mut clash_rules: Vec<String> = Vec::new();
    for rule_set in rulesets.rulesets.iter() {
        let ruleset = &rule_set.ruleset;
        if ruleset == "MATCH" {
            clash_rules.push(format!("{},{}", ruleset, rule_set.group))
        } else {
            let ruleset_content = match std::fs::read_to_string(&format!("clash/{}", ruleset)) {
                Ok(s) => s,
                Err(e) => {
                    let error = match e.kind() {
                        std::io::ErrorKind::NotFound => format!("{ruleset} is not found."),
                        _ => format!("read {ruleset} failed: {e}"),
                    };
                    return Err(error.into());
                }
            };

            let rules: Vec<String> = ruleset_content
                .lines()
                .filter(|line| !line.starts_with("#"))
                .map(|line| format!("{},{}", line, rule_set.group))
                .collect();
            clash_rules.extend(rules);
        }
    }

    let mut clash_rules_string = String::new();
    clash_rules_string.push_str("rules:\n");
    for rule in clash_rules.iter_mut() {
        let re = Regex::new("no-resolve").unwrap();

        if re.is_match(&rule) {
            let mut parts: Vec<&str> = rule.split(',').collect();
            parts.swap(2, 3);
            *rule = parts.join(",");
        }

        clash_rules_string.push_str(&format!("  - {}\n", rule));
    }

    Ok(clash_rules_string)

}

pub fn copy_directory<S: AsRef<Path>, D: AsRef<Path>>(
    source: S,
    destination: D,
) -> std::io::Result<()> {
    // 创建目标目录
    fs::create_dir_all(&destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = destination.as_ref().join(entry.file_name());

        if path.is_dir() {
            // 如果是目录，递归复制
            copy_directory(path, dest_path)?;
        } else {
            // 如果是文件，直接复制
            fs::copy(path, dest_path)?;
        }
    }

    Ok(())
}
