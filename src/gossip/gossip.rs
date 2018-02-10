use db::{self, Neighbour};
use futures::{Future, MapErr, Stream};
use futures::stream::ForEach;
use tokio::net::UdpFramed;
use statics::id;
use std::io;
use std::net::SocketAddr;
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
    pub fn init() -> io::Result<Self> {
        let reader = Caster::new()?.framed()?;
        let writer = Caster::new()?;

        Ok(Self {
            server: reader
                .for_each(&server as &ServerFn)
                .map_err(&error as &ErrorFn),
            writer: Arc::new(writer)
        })
    }
}

type ServerFn = (Fn((Option<Message>, SocketAddr)) -> io::Result<()>);
fn server(inbound: (Option<Message>, SocketAddr)) -> io::Result<()> {
    let (msg, addr) = inbound;

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
        let db = db::open::<Neighbour>();
        let n = if let Ok(Some(mut n)) = db.get(&msg.id) {
            n.seen();
            n
        } else {
            Neighbour::default()
        };
        let _ = db.set(&msg.id, &n);
        println!("Got a ping from {} ({})!\nFirst seen: {:?}\nLast Seen: {:?}",
            msg.id, addr, n.first_seen, n.last_seen
        );
    }

    println!("{:?}", msg);
    Ok(())
}

type ErrorFn = (Fn(io::Error) -> ());
fn error(err: io::Error) -> () {
    println!("Server error: {}", err);
    ()
}
