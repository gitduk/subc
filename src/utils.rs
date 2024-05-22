use base64::{engine::general_purpose, Engine as _};
use std::fs::OpenOptions;
use std::io::Write;

pub fn base64_decode(value: String) -> String {
    let value_vec = &general_purpose::STANDARD
        .decode(&value)
        .expect(&format!("Decode {value} error"));
    String::from_utf8(value_vec.to_owned()).unwrap()
}

pub fn base64_decode_no_pad(value: String) -> String {
    let value_vec = &general_purpose::STANDARD_NO_PAD
        .decode(&value)
        .expect(&format!("Decode {value} error"));
    String::from_utf8(value_vec.to_owned()).unwrap()
}

pub fn write_to_file(file_name: &str, content: &str) -> anyhow::Result<()> {
    // write content to ~/.config/clash/file_name
    // TODO ... optimize

    let xdg_dirs = xdg::BaseDirectories::with_prefix("clash").unwrap();
    let file_path = xdg_dirs
        .place_config_file(file_name)
        .expect("cannot create configuration directory");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    write!(&mut file, "{}", content)?;
    Ok(())
}
