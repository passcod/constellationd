use bytes::BytesMut;
use message::Message;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

use super::DatagramCodec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatastreamCodec(DatagramCodec);

impl Default for DatastreamCodec {
    fn default() -> Self {
        DatastreamCodec(DatagramCodec::default())
    }
}

impl Decoder for DatastreamCodec {
    type Item = io::Result<Message>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match DatagramCodec::default().decode(buf) {
            Ok(Some(Err(err))) => match err.kind() {
                io::ErrorKind::UnexpectedEof => Ok(None),
                _ => Ok(Some(Err(err)))
            },
            v @ _ => v
        }
    }
}

impl Encoder for DatastreamCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        DatagramCodec::default().encode(msg, buf)
    }
}
