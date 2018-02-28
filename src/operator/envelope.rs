use constants;
use rust_sodium::crypto::secretbox::{gen_nonce, open, Nonce, seal};
use serde_bytes;
use serde_cbor;
use statics;
use std::io;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Envelope<'a> {
    pub v: u8,
    pub key: &'a str,
    pub nonce: Nonce,

    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

impl<'a> Envelope<'a> {
    pub fn new(msg: &[u8]) -> Self {
        let nonce = gen_nonce();
        let body = seal(msg, &nonce, statics::secret());

        Envelope {
            v: constants::PROTOCOL_VERSION,
            key: statics::key(),
            nonce: nonce,
            body: body,
        }
    }

    pub fn open(&self) -> Result<Vec<u8>, io::Error> {
        open(&self.body, &self.nonce, statics::secret())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "decrypt fail"))
    }

    pub fn pack(&self) -> Result<Vec<u8>, serde_cbor::error::Error> {
        serde_cbor::to_vec(&self)
    }

    pub fn unpack(buf: &'a [u8]) -> Result<Option<Self>, io::Error> {
        match serde_cbor::from_slice(buf) {
            Err(err) => {
                if format!("{:?}", err).starts_with("ErrorImpl { code: EofWhileParsing") {
                    Ok(None) // incomplete input
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, err))
                }
            },
            Ok(e) => Ok(Some(e))
        }
    }

    pub fn check(&self) -> bool {
        if self.v != constants::PROTOCOL_VERSION {
            return false;
        }

        if self.key != statics::key() {
            return false;
        }

        true
    }
}
