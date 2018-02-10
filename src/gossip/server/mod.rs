use statics::id;
use std::io;
use super::{Message, MessageBody};

mod hello;

pub type ServerFn = Fn(Message) -> io::Result<()>;
pub fn server(msg: Message) -> io::Result<()> {
    // Ignore own messages
    if &msg.id == id() {
        return Ok(())
    }

    // Record hellos
    if let &MessageBody::Hello(ref hello) = &msg.body {
        hello::serve(&msg, hello);
    }

    println!("{:?}", msg);
    Ok(())
}
