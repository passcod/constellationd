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
use shine::Glow;
use message::Message;
use statics::id;
use std::time::Duration;
use tokio::executor::current_thread;

mod config;
mod constants;
mod db;
mod envelope;
mod errors;
mod shine;
mod keygen;
#[macro_use] mod macros;
mod message;
mod operator;
mod statics;

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

    let mut light = Glow::init().expect("Failed to initialise glow");
    light.sender.try_send(Message::hello()).expect("Failed to send hello");

    let mut ping_sender = light.sender.clone();
    let timer = tokio_timer::Timer::default();
    let pinger = timer.interval(Duration::new(10, 0)).for_each(move |_| {
        let _ = ping_sender.try_send(Message::hello());
        Ok(())
    });

    current_thread::run(|_| {
        current_thread::spawn(plumb!("pinger", pinger));
        current_thread::spawn(plumb!("shine.server", light.server));
        current_thread::spawn(plumb!("shine.writer", light.writer));
        current_thread::spawn(plumb!("operator", operator::server()));
    })
}
