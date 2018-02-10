use constants::*;
use futures::{Async, AsyncSink, Poll, Sink, StartSend, Stream};
use net2::UdpBuilder;
use net2::unix::UnixUdpBuilderExt;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use super::{GossipCodec, Message};
use tokio::net::{UdpFramed, UdpSocket as TokioUdp};
use tokio::reactor::Handle;
use tokio_io::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct Caster {
    cast: SocketAddr,
    socket: UdpFramed<GossipCodec>,
}

impl Caster {
    pub fn new() -> io::Result<Self> {
        let sock = UdpBuilder::new_v4()?
            .reuse_address(true)?
            .reuse_port(true)?
            .bind(BIND)?;

        sock.set_broadcast(true)?;
        sock.set_multicast_loop_v4(true)?;
        sock.set_multicast_ttl_v4(128)?;
        sock.join_multicast_v4(&MULTI.into(), &ANY.into())?;

        Ok(Self {
            cast: SocketAddr::from_str(CAST).unwrap(),
            socket: UdpFramed::new(
                TokioUdp::from_std(sock, &Handle::default())?,
                GossipCodec
            ),
        })
    }
}

impl Sink for Caster {
    type SinkItem = <GossipCodec as Encoder>::Item;
    type SinkError = <GossipCodec as Encoder>::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.socket.start_send((item, self.cast.clone())).map(|async| match async {
            AsyncSink::NotReady((item, _)) => AsyncSink::NotReady(item),
            AsyncSink::Ready => AsyncSink::Ready,
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.socket.poll_complete()
    }
}

impl Stream for Caster {
    type Item = Message;
    type Error = <GossipCodec as Decoder>::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.socket.poll().map(|async| match async {
            Async::Ready(Some((Some(item), _))) => Async::Ready(Some(item)),
            Async::Ready(Some((None, _))) => Async::Ready(None),
            Async::Ready(None) => Async::Ready(None),
            Async::NotReady => Async::NotReady,
        })
    }
}
