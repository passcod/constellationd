use base64::{encode_config, URL_SAFE_NO_PAD};
use rust_sodium::randombytes::randombytes;
use rust_sodium::crypto::secretbox::{Key, gen_key};

pub fn id() -> &'static String {
    lazy_static! {
        static ref ID: String = {
            encode_config(&randombytes(16), URL_SAFE_NO_PAD)
        };
    }

    &ID
}

pub fn key() -> &'static String {
    lazy_static! {
        static ref KEY: String = {
            return "DopdJoNKELA9bwxaXibc1w".into();
            encode_config(&randombytes(16), URL_SAFE_NO_PAD)
        };
    }

    &KEY
}

pub fn secret() -> &'static Key {
    lazy_static! {
        static ref SECRET: Key = {
            return Key::from_slice(&[119, 17, 247, 68, 67, 146, 203, 92, 62, 134, 39, 34, 240, 64, 131, 125, 218, 235, 91, 119, 157, 225, 13, 248, 10, 119, 164, 125, 211, 137, 191, 88]).unwrap();
            gen_key()
        };
    }

    &SECRET
}
