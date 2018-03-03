extern crate base64;
extern crate bytes;
extern crate futures;
extern crate itertools;
extern crate interfaces;
#[macro_use] extern crate lazy_static;
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
use gossip::Gossip;
use message::Message;
use statics::id;
use std::time::Duration;
use tokio::executor::current_thread;

mod config;
mod constants;
mod db;
mod envelope;
mod errors;
mod gossip;
mod keygen;
mod message;
// mod operator;
mod statics;

macro_rules! plumb {
    ($label:expr, $future:expr) => ({
        $future
        .map(|thing| {println!("{} map {:?}", $label, thing);})
        .map_err(|thing| {println!("{} err {:?}", $label, thing);})
    })
}

fn main() {
    println!("{} v{}\nID: {}\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        id()
    );

    if !rust_sodium::init() {
        panic!("Failed to initialise sodium");
    }

    keygen::main();

    let mut gossip = Gossip::init().expect("Failed to initialise gossip");
    gossip.sender.try_send(Message::hello()).expect("Failed to send hello");

    let mut ping_sender = gossip.sender.clone();
    let timer = tokio_timer::Timer::default();
    let pinger = timer.interval(Duration::new(10, 0)).for_each(move |_| {
        let _ = ping_sender.try_send(Message::hello());
        Ok(())
    });

    current_thread::run(|_| {
        current_thread::spawn(plumb!("pinger", pinger));
        current_thread::spawn(plumb!("gossip.server", gossip.server));
        current_thread::spawn(plumb!("gossip.writer", gossip.writer));
        // current_thread::spawn(plumb!("operator", operator::server()));
    })
}
