use constants;
use rust_sodium::crypto::secretbox::{gen_nonce, open, Nonce, seal};
use serde_bytes;
use serde_cbor;
use statics;
use super::Message;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Envelope {
    pub v: u8,
    pub key: String,
    pub nonce: Nonce,

    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

impl Envelope {
    pub fn new(msg: &Message) -> Self {
        let nonce = gen_nonce();
        let ser = serde_cbor::to_vec(&msg).expect("Unable to encode message");
        let body = seal(&ser, &nonce, statics::secret());

        Envelope {
            v: constants::PROTOCOL_VERSION,
            key: statics::key().clone(),
            nonce: nonce,
            body: body,
        }
    }

    pub fn open(&self) -> Result<Option<Message>, serde_cbor::error::Error> {
        match open(&self.body, &self.nonce, statics::secret()) {
            Err(_) => Ok(None),
            Ok(msg) => serde_cbor::from_slice(&msg).map(|m| Some(m))
        }
    }

    pub fn pack(&self) -> Result<Vec<u8>, serde_cbor::error::Error> {
        serde_cbor::to_vec(&self)
    }

    pub fn unpack(buf: &[u8]) -> Option<Self> {
        match serde_cbor::from_slice(buf) {
            Err(err) => {
                println!("Bad cbor: {:?}\n{:?}", buf, err);
                None
            },
            Ok(e) => Some(e)
        }
    }

    pub fn check(&self) -> bool {
        if self.v != constants::PROTOCOL_VERSION {
            return false;
        }

        if &self.key != statics::key() {
            return false;
        }

        true
    }
}
