use serde_json;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Config {
    pub key: String,
    pub secret: Vec<u8>,
}

impl Config {
    pub fn from_file() -> Option<Self> {
        let mut file = match File::open("constellationd.json") {
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
}
