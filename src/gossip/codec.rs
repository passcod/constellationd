use constants::*;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::net::UdpCodec;

use super::{Envelope, Message};

pub struct GossipCodec;

impl UdpCodec for GossipCodec {
    type In = Option<Message>;
    type Out = Message;

    fn decode(&mut self, _: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        let env = match Envelope::unpack(buf) {
            None => return Ok(None),
            Some(e) => e
        };

        if ! env.check() {
            println!("Bad metadata: {:?}", env);
            return Ok(None)
        }

        match env.open() {
            Err(err) => {
                println!("Bad json: {:?}\n{:?}", buf, err);
                Ok(None)
            },
            Ok(None) => {
                println!("Bad encryption: {:?}", buf);
                Ok(None)
            },
            Ok(Some(m)) => Ok(Some(m))
        }
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        let ser = Envelope::new(&msg).pack().expect("Unable to encode message");
        buf.extend(ser);
        SocketAddr::from_str(CAST).unwrap()
    }
}
