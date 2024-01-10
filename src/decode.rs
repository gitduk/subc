use base64::{engine::general_purpose, Engine as _};

pub fn base64_decode(value: String) -> String {
    let value_vec = &general_purpose::STANDARD
        .decode(&value)
        .expect(&format!("decode {value} error"));
    String::from_utf8(value_vec.to_owned()).unwrap()
}

pub fn base64_decode_no_pad(value: String) -> String {
    let value_vec = &general_purpose::STANDARD_NO_PAD
        .decode(&value)
        .expect(&format!("decode {value} error"));
    String::from_utf8(value_vec.to_owned()).unwrap()
}
