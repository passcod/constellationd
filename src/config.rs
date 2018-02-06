use base64::{encode_config, URL_SAFE_NO_PAD};
use constants;
use rust_sodium::randombytes::randombytes;
use rust_sodium::crypto::secretbox::{gen_key};
use serde_json;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Config {
    pub key: String,
    pub secret: Vec<u8>,
}

impl Config {
    pub fn from_file() -> Option<Self> {
        let mut file = match File::open(constants::CONFIG_FILE) {
            Err(_) => return None,
            Ok(f) => f
        };

        let mut conf = String::new();
        if let Err(_) = file.read_to_string(&mut conf) {
            return None;
        }

        match serde_json::from_slice(conf.as_bytes()) {
            Ok(c) => c,
            Err(_) => None
        }
    }

    pub fn from_embed() -> Option<Self> {
        let conf = include_str!("config.json");
        if conf.len() < 107 {
            None
        } else {
            match serde_json::from_slice(conf.as_bytes()) {
                Ok(c) => c,
                Err(_) => None
            }
        }
    }

    pub fn generate() -> Self {
        Config {
            key: encode_config(&randombytes(16), URL_SAFE_NO_PAD),
            secret: gen_key()[..].to_vec()
        }
    }

    pub fn to_file(&self) {
        let ser = serde_json::to_vec(&self).expect("Unable to encode config");
        let mut file = File::create(constants::CONFIG_FILE).expect("Couldn't open config file");
        file.write_all(&ser).expect("Couldn't write config to file");
    }
}
