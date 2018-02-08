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
extern crate tokio_timer;

use futures::{Future, Stream};
use gossip::{Caster, Message};
use statics::id;
use std::time::Duration;
use tokio_core::reactor::Core;

mod config;
mod constants;
mod gossip;
mod keygen;
mod statics;

#[derive(Debug)]
enum StreamError {
    Io(std::io::Error),
    Timer(tokio_timer::TimerError),
}

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

    let timer = tokio_timer::Timer::default();

    // Send pings
    let pinger = timer.interval(Duration::new(10, 0)).for_each(|()| {
        let _ = writer.send(&Message::ping());
        Ok(())
    }).map_err(|err| {
        println!("Timer error: {}", err);
        StreamError::Timer(err)
    });

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

        // Record pings
        if msg.kind.is_ping() {
            println!("Got a ping from {}!", msg.id);
        }

        println!("{:?}", msg);
        Ok(())
    }).map_err(|err| {
        println!("Server error: {}", err);
        StreamError::Io(err)
    });

    if let Err(_) = core.run(
        server.select(pinger)
    ) {
        println!("Failed to start UDP server");
    }
}
