use base64::{encode_config, URL_SAFE_NO_PAD};
use config::Config;
use rust_sodium::randombytes::randombytes;
use rust_sodium::crypto::secretbox::Key;

/// The unique ID of this particular instance of this agent.
pub fn id() -> &'static String {
    lazy_static! {
        static ref ID: String = {
            encode_config(&randombytes(16), URL_SAFE_NO_PAD)
        };
    }

    &ID
}

/// The cluster key, identifying the cluster for disambiguation.
///
/// Multiple clusters can cohabit on the same network without interfering
/// with each other. Technically, that could be possible entirely via
/// encryption, but for efficiency and inspection ease, we use a "key" that
/// is attached to every gossip message in plain, and used to filter messages
/// (i.e. discard those that don't match).
pub fn key() -> &'static String {
    lazy_static! {
        static ref KEY: String = {
            config().key.clone()
        };
    }

    &KEY
}

/// The cluster secret, encrypting all gossip.
pub fn secret() -> &'static Key {
    lazy_static! {
        static ref SECRET: Key = {
            Key::from_slice(&config().secret).expect("Secret corrupted or missing.")
        };
    }

    &SECRET
}

/// The config, loaded once.
///
/// Loads either from embed or from file, in this order.
pub fn config() -> &'static Config {
    lazy_static! {
        static ref CONFIG: Config = {
            Config::from_embed()
                .or_else(|| Config::from_file())
                .expect("No config found, abort.")
        };
    }

    &CONFIG
}
