use errors::IgnoredIoError;
use futures::{Sink, Stream};
use futures::sink::SinkFromErr;
use futures::stream::{Forward, ForEach, SplitSink, SplitStream};
use futures::sync::mpsc::{channel, Sender, Receiver};
use message::Message;
use std::io;
use super::Caster;
use super::server::{server, ServerFn};

pub struct Gossip<'a> {
    pub server: ForEach<
        SplitStream<Caster>,
        &'a ServerFn,
        io::Result<()>
    >,
    pub writer: Forward<
        Receiver<Message>,
        SinkFromErr<
            SplitSink<Caster>,
            IgnoredIoError
        >
    >,
    pub sender: Sender<Message>,
}

impl<'a> Gossip<'a> {
    pub fn init() -> io::Result<Self> {
        let (writer, reader) = Caster::new()?.split();
        let (tx, rx) = channel(100);

        Ok(Self {
            server: reader.for_each(&server as &ServerFn),
            writer: rx.forward(writer.sink_from_err::<IgnoredIoError>()),
            sender: tx,
        })
    }
}
