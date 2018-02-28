use bytes::BytesMut;
use errors::SendError;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

use super::envelope::Envelope;

#[inline]
fn io_error(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperatorCodec {
    header_len: Option<u8>,
    payload_len: Option<usize>,
}

impl Default for OperatorCodec {
    fn default() -> Self {
        Self {
            header_len: None,
            payload_len: None,
        }
    }
}

impl Decoder for OperatorCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None) // request more data
        }

        let v = buf.get(0);
        if v.is_none() { return Ok(None) }
        let v = v.unwrap() as &u8;
        if v != &0 { return Err(io_error("bad version".into())) }

        let hlen = buf.get(1);
        if hlen.is_none() { return Ok(None) }
        let hlen = hlen.unwrap() as &u8;
        if hlen == &0 { return Err(io_error("bad header length".into())) }

        self.header_len = Some(*hlen);

        let env = match Envelope::unpack(buf)? {
            None => return Ok(None), // incomplete input
            Some(e) => e
        };

        if ! env.check() {
            return Err(io_error(format!("Bad metadata: {:?}", env)))
        }

        env.open().map(|b| Some(b))
    }
}

impl Encoder for OperatorCodec {
    type Item = Vec<u8>;
    type Error = SendError;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let ser = Envelope::new(&msg).pack()?;
        buf.extend(ser);
        Ok(())
    }
}
