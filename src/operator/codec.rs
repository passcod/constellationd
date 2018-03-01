use bytes::BytesMut;
use constants;
use errors::SendError;
use rust_sodium::crypto::secretbox::{gen_nonce, open, Nonce, seal};
use serde_cbor;
use statics;
use std::slice::Iter;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

use super::envelope::Envelope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorCodec {
    header_len: Option<u8>,
    nonce: Option<Nonce>,
    payload_len: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct Header (String, Nonce, usize);

impl Default for OperatorCodec {
    fn default() -> Self {
        Self {
            header_len: None,
            nonce: None,
            payload_len: None,
        }
    }
}

fn io_error(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

fn take_version(buf: &BytesMut) -> Result<Option<()>, io::Error> {
    match buf.get(0) {
        None => Ok(None),
        Some(&constants::PROTOCOL_VERSION) => Ok(Some(())),
        Some(v) => {
            buf.clone().advance(1);
            Err(io_error(format!("bad version: {}", v)))
        },
    }
}

fn take_header_len(buf: &BytesMut) -> Result<Option<u8>, io::Error> {
    match buf.get(1) {
        None => Ok(None),
        Some(&0) => {
            buf.clone().advance(2);
            Err(io_error("zero length header".into()))
        },
        Some(h) => Ok(Some(*h)),
    }
}

fn take_header(buf: &BytesMut, length: u8) -> Result<Option<Header>, io::Error> {
    let length = length as usize;
    let hbuf = &buf[2..(length + 2)];
    if hbuf.len() != length {
        return Ok(None);
    }

    return match serde_cbor::from_slice(hbuf) {
        Err(err) => {
            let serr = format!("{:?}", err);
            if serr.starts_with("ErrorImpl { code: EofWhileParsing") {
                Ok(None) // incomplete input
            } else {
                buf.clone().advance(2 + length);
                Err(io_error(serr))
            }
        },
        Ok(e) => Ok(Some(e))
    };
}

impl Decoder for OperatorCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None) // request more data
        }

        if self.payload_len.is_none() {
            if self.header_len.is_none() {
                if take_version(buf)?.is_none() {
                    return Ok(None);
                }

                if let Some(h) = take_header_len(buf)? {
                    self.header_len = Some(h);
                } else {
                    return Ok(None);
                }
            }

            if let Some(header) = take_header(buf, self.header_len.unwrap())? {
                let total = 2 + (self.header_len.unwrap() as usize) + header.2;

                if &header.0 != statics::key() {
                    buf.clone().advance(total);
                    return Err(io_error("invalid key".into()));
                }

                if header.2 == 0 {
                    buf.clone().advance(total);
                    return Err(io_error("zero length payload".into()));
                }

                self.nonce = Some(header.1);
                self.payload_len = Some(header.2);
            } else {
                return Ok(None);
            }
        }

        let start = 2 + (self.header_len.unwrap() as usize);
        let length = self.payload_len.unwrap();
        let total_expected = start + length;
        if buf.len() < total_expected {
            return Ok(None);
        }

        let pbuf = &buf[start..(start + length)];
        if pbuf.len() < length {
            return Ok(None); // just checking...
        }

        match open(pbuf, &self.nonce.unwrap(), statics::secret()) {
            Err(_) => {
                buf.clone().advance(total_expected);
                Err(io_error("bad payload encryption".into()))
            },
            Ok(payload) => {
                buf.clone().advance(total_expected);
                Ok(Some(payload))
            }
        }
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
