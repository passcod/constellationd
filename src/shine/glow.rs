use errors::IgnoredIoError;
use futures::{Sink, Stream};
use futures::sink::SinkFromErr;
use futures::stream::{FilterMap, Forward, ForEach, SplitSink, SplitStream};
use futures::sync::mpsc::{channel, Sender, Receiver};
use message::Message;
use std::io;
use super::Caster;
use super::server::{server, ServerFn};

pub struct Glow<'a> {
    pub server: ForEach<
        FilterMap<
            SplitStream<Caster>,
            &'a FilterFn
        >,
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

pub type FilterFn = Fn(io::Result<Message>) -> Option<Message>;
fn filter(msg: io::Result<Message>) -> Option<Message> {
    match msg {
        Ok(m) => {
            // println!("\x1b[0;32mGood incoming:\x1b[0m {:?}", m);
            Some(m)
        },
        Err(e) => {
            println!("\x1b[0;31mBad incoming:\x1b[0m {}", e);
            None
        }
    }
}

impl<'a> Glow<'a> {
    pub fn init() -> io::Result<Self> {
        let (writer, reader) = Caster::new()?.split();
        let (tx, rx) = channel(100);

        Ok(Self {
            server: reader.filter_map(&filter as &FilterFn).for_each(&server as &ServerFn),
            writer: rx.forward(writer.sink_from_err::<IgnoredIoError>()),
            sender: tx,
        })
    }
}
