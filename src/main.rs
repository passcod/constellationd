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

use futures::Stream;
use gossip::{Caster, Message};
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

    let reader = Caster::new().expect("Failed to bind UDP")
        .framed(&handle).expect("Failed to frame");

    let writer = Caster::new().expect("Failed to bind UDP");
    writer.send(&Message::hello()).expect("Failed to send hello");

    let server = reader.for_each(|msg| {
        // Ignore empty (errored) messages
        let msg = match msg {
            None => return Ok(()),
            Some(m) => m
        };

        // Ignore own messages
        if &msg.id == id() {
            return Ok(())
        }

        // Answer pings
        if msg.kind.is_ping() {
            if let Err(err) = writer.send(
                &Message::pong(msg.seq.unwrap())
            ) {
                println!("Failed send: {:?}", err);
            }
        }

        println!("{:?}", msg);
        Ok(())
    });

    core.run(server).expect("Failed to start UDP server");
}
