use constants::*;
use net2::UdpBuilder;
use net2::unix::UnixUdpBuilderExt;
use serde_cbor::error::Error as CborError;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use super::Envelope;
use super::GossipCodec;
use super::Message;
use tokio_core::net::{UdpFramed, UdpSocket as TokioUdp};
use tokio_core::reactor::Handle;

#[derive(Debug)]
pub enum SendError {
    Io(io::Error),
    Encode(CborError),
}

#[derive(Debug)]
pub struct Caster {
    cast: SocketAddr,
    socket: UdpSocket,
}

impl Caster {
    pub fn new() -> io::Result<Self> {
        let sock = UdpBuilder::new_v4()?
            .reuse_address(true)?
            .reuse_port(true)?
            .bind(BIND)?;

        sock.set_broadcast(true)?;
        sock.set_multicast_loop_v4(true)?;
        sock.set_multicast_ttl_v4(1)?; // Set higher to reach outside local
        sock.join_multicast_v4(&MULTI.into(), &ANY.into())?;

        Ok(Self {
            cast: SocketAddr::from_str(CAST).unwrap(),
            socket: sock,
        })
    }

    pub fn framed(self, handle: &Handle) -> io::Result<UdpFramed<GossipCodec>> {
        TokioUdp::from_socket(self.socket, handle)
        .map(|s| s.framed(GossipCodec))
    }

    pub fn send(&self, message: &Message) -> Result<usize, SendError> {
        Envelope::new(message)
        .pack().map_err(|err| SendError::Encode(err))
        .and_then(|buf|
            self.socket.send_to(&buf, &self.cast)
            .map_err(|err| SendError::Io(err))
        )
    }
}
