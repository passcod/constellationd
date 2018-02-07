extern crate base64;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate net2;
extern crate rust_sodium;
extern crate serde;
extern crate serde_bytes;
extern crate serde_cbor;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

use futures::{Sink, Stream};
use gossip::{message, Message};
use statics::id;
use tokio_core::reactor::Core;

mod config;
mod constants;
mod gossip;
mod keygen;
mod statics;

fn main() {
    println!("{} v{}\nID: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        id()
    );

    if !rust_sodium::init() {
        panic!("Failed to initialise sodium");
    }

    keygen::main();

    let mut core = Core::new().expect("Failed to initialise event loop");
    let handle = core.handle();

    let (mut writer, reader) = gossip::udp(&handle).expect("Failed to bind UDP")
        .framed(gossip::GossipCodec).split();

    let server = reader.for_each(|msg| {
        let msg = match msg {
            None => return Ok(()),
            Some(m) => m
        };

        // Ignore own messages
        if &msg.id == id() { return Ok(()) }

        if msg.kind == message::Kind::Ping {
            if let Err(err) = writer.start_send(Message::new(message::Kind::Pong)) {
                println!("Failed send: {:?}", err);
            }
        }

        println!("{:?}", msg);
        Ok(())
    });

    core.run(server).expect("Failed to start UDP server");
}
