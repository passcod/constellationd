use statics::id;
use std::io;
use message::{Message, Body};

mod hello;

pub type ServerFn = Fn(Message) -> io::Result<()>;
pub fn server(msg: Message) -> io::Result<()> {
    // Ignore own messages
    if &msg.id == id() {
        return Ok(())
    }

    // Record hellos
    if let &Body::Hello(ref hello) = &msg.body {
        hello::serve(&msg, hello);
    }

    println!("{:?}", msg);
    Ok(())
}
