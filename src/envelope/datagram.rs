use bytes::{BufMut, BytesMut};
use constants;
use message::Message;
use rust_sodium::crypto::secretbox::{gen_nonce, open, Nonce, seal};
use serde_cbor;
use statics;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatagramCodec {
    header_len: Option<u8>,
    nonce: Option<Nonce>,
    payload_len: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct Header (String, Nonce, usize);

impl Default for DatagramCodec {
    fn default() -> Self {
        Self {
            header_len: None,
            nonce: None,
            payload_len: None,
        }
    }
}

fn io_error(message: String) -> io::Error {
    println!("io_error {}", message);
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

    match serde_cbor::from_slice(hbuf) {
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
    }
}

macro_rules! kind_try {
    ($e:expr) => {match $e {
        Err(e) => return Ok(Some(Err(e))),
        Ok(o) => o
    }};
}

macro_rules! need_more {
    ($msg:expr) => {
        return Ok(Some(Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("Need more because: {}", $msg)
        ))));
    };
}

impl Decoder for DatagramCodec {
    type Item = io::Result<Message>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            need_more!("empty buffer");
        }

        if self.payload_len.is_none() {
            if self.header_len.is_none() {
                if kind_try!(take_version(buf)).is_none() {
                    need_more!("no version");
                }

                if let Some(h) = kind_try!(take_header_len(buf)) {
                    self.header_len = Some(h);
                } else {
                    need_more!("no header length");
                }
            }

            if let Some(header) = kind_try!(take_header(buf, self.header_len.unwrap())) {
                let total = 2 + (self.header_len.unwrap() as usize) + header.2;

                if &header.0 != statics::key() {
                    buf.clone().advance(total);
                    return Ok(Some(Err(io_error("invalid key".into()))));
                }

                if header.2 == 0 {
                    buf.clone().advance(total);
                    return Ok(Some(Err(io_error("zero length payload".into()))));
                }

                self.nonce = Some(header.1);
                self.payload_len = Some(header.2);
            } else {
                need_more!("incomplete header");
            }
        }

        let start = 2 + (self.header_len.unwrap() as usize);
        let length = self.payload_len.unwrap();
        let total_expected = start + length;
        if buf.len() < total_expected {
            need_more!("incomplete header");
        }

        let pbuf = &buf[start..(start + length)];
        if pbuf.len() < length {
            need_more!("incomplete header (just checking...)");
        }

        let payload = match open(pbuf, &self.nonce.unwrap(), statics::secret()) {
            Err(_) => {
                buf.clone().advance(total_expected);
                return Ok(Some(Err(io_error("bad payload encryption".into()))));
            },
            Ok(payload) => {
                buf.clone().advance(total_expected);
                payload
            }
        };

        match serde_cbor::from_slice(&payload) {
            Err(err) => Ok(Some(Err(io_error(format!("{}", err))))),
            Ok(m) => Ok(Some(Ok(m)))
        }
    }
}

impl Encoder for DatagramCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let msg_packed = serde_cbor::to_vec(&msg).map_err(
            |err| io::Error::new(io::ErrorKind::Other, format!("{}", err))
        )?;

        let nonce = gen_nonce();
        let payload = seal(&msg_packed, &nonce, statics::secret());

        let header = Header(statics::key().into(), nonce, payload.len());
        let header_packed = serde_cbor::to_vec(&header).map_err(
            |err| io::Error::new(io::ErrorKind::Other, format!("{}", err))
        )?;

        let header_len = header_packed.len();
        let total = 2 + header_len + header.2;

        let cap = buf.capacity();
        if cap < total {
            buf.reserve(total - cap);
        }

        assert!(buf.remaining_mut() >= total);

        buf.put_u8(constants::PROTOCOL_VERSION);
        buf.put_u8(header_len as u8);
        buf.put(header_packed);
        buf.put(payload);

        Ok(())
    }
}
