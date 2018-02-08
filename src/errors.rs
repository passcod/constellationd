use serde_cbor::error::Error as CborError;
use std::io::Error as IoError;
use tokio_timer::TimerError;

#[derive(Debug)]
pub enum StreamError {
    Io(IoError),
    Timer(TimerError),
}

#[derive(Debug)]
pub enum SendError {
    Io(IoError),
    Encode(CborError),
}
