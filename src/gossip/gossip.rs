use db::{self, Neighbour};
use errors::StreamError;
use futures::{Future, MapErr, Stream};
use futures::stream::ForEach;
use tokio_core::reactor::Handle;
use tokio_core::net::UdpFramed;
use statics::id;
use std::io;
use std::sync::Arc;
use super::{Caster, GossipCodec, Message};

pub struct Gossip<'a> {
    pub server: MapErr<ForEach<
        UdpFramed<GossipCodec>,
        &'a ServerFn,
        io::Result<()>
    >, &'a ErrorFn>,
    pub writer: Arc<Caster>,
}

impl<'a> Gossip<'a> {
    pub fn init(handle: &Handle) -> io::Result<Self> {
        let reader = Caster::new()?.framed(handle)?;
        let writer = Caster::new()?;

        Ok(Self {
            server: reader
                .for_each(&server as &ServerFn)
                .map_err(&error as &ErrorFn),
            writer: Arc::new(writer)
        })
    }
}

type ServerFn = (Fn(Option<Message>) -> io::Result<()>);
fn server(msg: Option<Message>) -> io::Result<()> {
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
        let db = db::rw().unwrap();
        let _ = db.set(&msg.id, &Neighbour::default());
        println!("Got a ping from {}!", msg.id);
    }

    println!("{:?}", msg);
    Ok(())
}

type ErrorFn = (Fn(io::Error) -> StreamError);
fn error(err: io::Error) -> StreamError {
    println!("Server error: {}", err);
    StreamError::Io(err)
}
