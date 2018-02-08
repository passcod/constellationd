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
use gossip::{Gossip, Message};
use statics::id;
use std::time::Duration;
use tokio_core::reactor::Core;

mod config;
mod constants;
mod errors;
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
    let gossip = Gossip::init(&handle).expect("Failed to initialise gossip");
    let writer = gossip.writer.clone();

    let timer = tokio_timer::Timer::default();

    // Send pings
    let pinger = timer.interval(Duration::new(10, 0)).for_each(|()| {
        let _ = writer.send(&Message::ping());
        Ok(())
    }).map_err(|err| {
        println!("Timer error: {}", err);
        errors::StreamError::Timer(err)
    });

    writer.send(&Message::hello()).expect("Failed to send hello");
    if let Err(_) = core.run(
        pinger.select(gossip.server)
    ) {
        println!("Failed to start UDP server");
    }
}
