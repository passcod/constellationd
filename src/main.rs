extern crate base64;
extern crate bytes;
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
extern crate sled;
extern crate tempdir;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_timer;

use futures::{Future, Stream};
use gossip::{Gossip, Message};
use statics::id;
use std::time::Duration;
use tokio::executor::current_thread;

mod config;
mod constants;
mod db;
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

    let gossip = Gossip::init().expect("Failed to initialise gossip");
    let writer = gossip.writer.clone();
    writer.send(&Message::hello()).expect("Failed to send hello");

    let timer = tokio_timer::Timer::default();
    let pinger = timer.interval(Duration::new(10, 0))
    .for_each(move |_| {
        let _ = writer.send(&Message::ping());
        Ok(())
    }).map_err(|err| {
        println!("Timer error: {}", err);
        ()
    });

    current_thread::run(|_| {
        current_thread::spawn(pinger);
        current_thread::spawn(gossip.server);
    })
}
