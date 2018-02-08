use std::io::Error as IoError;
use tokio_timer::TimerError;

#[derive(Debug)]
pub enum StreamError {
    Io(IoError),
    Timer(TimerError),
}
