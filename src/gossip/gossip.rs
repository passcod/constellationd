use db::{self, Neighbour};
use futures::Stream;
use futures::stream::{Forward, ForEach, SplitSink, SplitStream};
use futures::sync::mpsc::{channel, Sender, Receiver};
use statics::id;
use std::io;
use super::{Caster, Message};

pub struct Gossip<'a> {
    pub server: ForEach<
        SplitStream<Caster>,
        &'a ServerFn,
        io::Result<()>
    >,
    pub writer: Forward<
        Receiver<Message>,
        SplitSink<Caster>
    >,
    pub sender: Sender<Message>,
}

impl<'a> Gossip<'a> {
    pub fn init() -> io::Result<Self> {
        let (writer, reader) = Caster::new()?.split();
        let (tx, rx) = channel(100);

        Ok(Self {
            server: reader.for_each(&server as &ServerFn),
            writer: rx.forward(writer),
            sender: tx,
        })
    }
}

type ServerFn = Fn(Message) -> io::Result<()>;
fn server(msg: Message) -> io::Result<()> {
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
        println!("Got a ping from {}!\nFirst seen: {:?}\nLast Seen: {:?}",
            msg.id, n.first_seen, n.last_seen
        );
    }

    println!("{:?}", msg);
    Ok(())
}
