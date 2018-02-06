use base64::{encode_config, URL_SAFE_NO_PAD};
use rust_sodium::randombytes::randombytes;

pub fn id() -> &'static String {
    lazy_static! {
        static ref ID: String = {
            encode_config(&randombytes(16), URL_SAFE_NO_PAD)
        };
    }

    &ID
}
