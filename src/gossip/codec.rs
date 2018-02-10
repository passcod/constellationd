use bytes::BytesMut;
use errors::SendError;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

use super::{Envelope, Message};

pub struct GossipCodec;

impl Decoder for GossipCodec {
    type Item = Option<Message>;
    type Error = io::Error;

    // Err() -- not happening, we'll absorb/ignore all errors to never break the stream
    // Ok(None) -- request more data (buffer empty or incomplete)
    // Ok(Some(None)) -- some error happened and we ignore it
    // Ok(Some(Some(Message))) -- got a Message!

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None) // request more data
        }

        let env = match Envelope::unpack(buf) {
            Err(_) => return Ok(None), // incomplete input
            Ok(None) => return Ok(Some(None)), // nothing of note
            Ok(Some(e)) => e
        };

        if ! env.check() {
            println!("Bad metadata: {:?}", env);
            return Ok(Some(None))
        }

        match env.open() {
            Err(err) => {
                println!("Bad json: {:?}\n{:?}", buf, err);
                Ok(Some(None))
            },
            Ok(None) => {
                println!("Bad encryption: {:?}", buf);
                Ok(Some(None))
            },
            Ok(Some(m)) => Ok(Some(Some(m)))
        }
    }
}

impl Encoder for GossipCodec {
    type Item = Message;
    type Error = SendError;

    // Err() -- encoding errors are fatal! they should not happen
    // Ok(()) -- all good, let's send this

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let ser = Envelope::new(&msg).pack()?;
        buf.extend(ser);
        Ok(())
    }
}
