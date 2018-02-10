use serde_cbor::error::Error as CborError;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum SendError {
    Io(IoError),
    Encode(CborError),
}

impl From<SendError> for () {
    fn from(_: SendError) {}
}

impl From<IoError> for SendError {
    fn from(err: IoError) -> Self {
        SendError::Io(err)
    }
}

impl From<CborError> for SendError {
    fn from(err: CborError) -> Self {
        SendError::Encode(err)
    }
}
