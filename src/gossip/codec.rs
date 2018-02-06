use constants::*;
use serde_json;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::net::UdpCodec;

use super::Message;

pub struct GossipCodec;

impl UdpCodec for GossipCodec {
    type In = Option<Message>;
    type Out = Message;

    fn decode(&mut self, _: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        serde_json::from_slice(buf)
            .or_else(|err| {
                println!("Bad message: {:?}\n{:?}", buf, err);
                Ok(None)
            })
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        let ser = serde_json::to_vec(&msg).expect("Unable to encode message");
        buf.extend(ser);
        SocketAddr::from_str(CAST).unwrap()
    }
}
