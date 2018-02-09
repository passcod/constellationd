use config::Config;
use constants;
use std::env;
use std::process::exit;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] != "--keygen" {
        return
    }

    if Config::from_embed().is_some() {
        println!("Embedded config present, not proceeding.");
        exit(4);
    }

    if Config::from_file().is_some() {
        println!("Config file already present, not proceeding.");
        exit(3);
    }

    println!("Writing new config to {}", constants::CONFIG_FILE);
    Config::default().to_file();
    exit(0);
}
