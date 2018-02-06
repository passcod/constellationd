use constants;
use rmp_serde;
use rust_sodium::crypto::secretbox::{gen_nonce, open, Nonce, seal};
use serde_json;
use statics;
use super::Message;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Envelope {
    pub v: u8,
    pub key: String,
    pub nonce: Nonce,
    pub body: Vec<u8>, // TODO: serialize as byte-array
}

impl Envelope {
    pub fn new(msg: &Message) -> Self {
        let nonce = gen_nonce();
        let ser = serde_json::to_vec(&msg).expect("Unable to encode message");
        let body = seal(&ser, &nonce, statics::secret());

        Envelope {
            v: constants::PROTOCOL_VERSION,
            key: statics::key().clone(),
            nonce: nonce,
            body: body,
        }
    }

    pub fn open(&self) -> Result<Option<Message>, serde_json::Error> {
        match open(&self.body, &self.nonce, statics::secret()) {
            Err(_) => Ok(None),
            Ok(msg) => serde_json::from_slice(&msg).map(|m| Some(m))
        }
    }

    pub fn pack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(&self)
    }

    pub fn unpack(buf: &[u8]) -> Option<Self> {
        match rmp_serde::from_slice(buf) {
            Err(err) => {
                println!("Bad msgpack: {:?}\n{:?}", buf, err);
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
