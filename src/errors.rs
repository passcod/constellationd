use std::io::Error as IoError;

#[derive(Debug)]
pub struct IgnoredIoError(IoError);

impl From<IgnoredIoError> for () {
    fn from(err: IgnoredIoError) {
        println!("ignoring error: {}", err.0);
    }
}

impl From<IoError> for IgnoredIoError {
    fn from(err: IoError) -> Self {
        IgnoredIoError(err)
    }
}
