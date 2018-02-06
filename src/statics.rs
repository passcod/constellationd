use base64::{encode_config, URL_SAFE_NO_PAD};
use rust_sodium::randombytes::randombytes;
use rust_sodium::crypto::secretbox::{Key, gen_key};

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
            // TODO get from file
            return "DopdJoNKELA9bwxaXibc1w".into();
            encode_config(&randombytes(16), URL_SAFE_NO_PAD)
        };
    }

    &KEY
}

/// The cluster secret, encrypting all gossip.
pub fn secret() -> &'static Key {
    lazy_static! {
        static ref SECRET: Key = {
            // TODO get from file
            return Key::from_slice(&[119, 17, 247, 68, 67, 146, 203, 92, 62, 134, 39, 34, 240, 64, 131, 125, 218, 235, 91, 119, 157, 225, 13, 248, 10, 119, 164, 125, 211, 137, 191, 88]).unwrap();
            gen_key()
        };
    }

    &SECRET
}
