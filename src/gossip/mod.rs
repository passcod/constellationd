use constants::*;
use net2::UdpBuilder;
use net2::unix::UnixUdpBuilderExt;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Handle;

pub use self::codec::GossipCodec;
pub use self::message::Message;
pub use self::envelope::Envelope;

mod codec;
mod envelope;
mod message;

pub fn udp(handle: &Handle) -> io::Result<UdpSocket> {
    let sock = UdpBuilder::new_v4()?
        .reuse_address(true)?
        .reuse_port(true)?
        .bind(BIND)?;

    sock.set_broadcast(true)?;
    sock.set_multicast_loop_v4(true)?;
    sock.set_multicast_ttl_v4(1)?; // Set higher to reach outside local
    sock.join_multicast_v4(&MULTI.into(), &ANY.into())?;

    // test message buffered by kernel and received immediately
    // by ourselves so we can check the tokio stack works
    let msg = Envelope::new(&Message::new(None)).pack().unwrap();
    sock.send_to(
        &msg,
        &SocketAddr::from_str(CAST).unwrap()
    ).expect("Failed to send");

    UdpSocket::from_socket(sock, handle)
}
