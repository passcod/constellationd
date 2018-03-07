use constants::BIND;
use errors::IgnoredIoError;
use futures::{Future, IntoFuture, MapErr, Sink, Stream};
use futures::stream::{ForEach};
use futures::sync::mpsc::{channel, Sender, Receiver};
use message::Message;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::executor::current_thread;
use tokio::net::{Incoming, TcpListener, TcpStream};
use tokio_io::AsyncRead;

use envelope::DatastreamCodec;

pub fn server<'a>() -> MapErr<ForEach<
    Incoming,
    &'a ServerFn,
    io::Result<()>
>, &'a ErrorFn> {
    let addr = SocketAddr::from_str(BIND).unwrap();
    let tcp = TcpListener::bind(&addr).unwrap();

    tcp.incoming()
    .for_each(&handle as &ServerFn)
    .map_err(&error as &ErrorFn)
}

type ServerFn = Fn(TcpStream) -> io::Result<()>;
fn handle(tcp: TcpStream) -> io::Result<()> {
    println!("Got a connection");

    let (writer, reader) = tcp.framed(DatastreamCodec::default()).split();
    let (tx, rx) = channel(10);

    let sink = rx.forward(writer.sink_from_err::<IgnoredIoError>());
    current_thread::spawn(plumb!("tcp.write", sink));

    let mut sender = tx.clone();
    let conn = reader.filter_map(|res| match res {
        Ok(m) => Some(m),
        Err(e) => {
            println!("Bad message: {:?}", e);
            None
        }
    }).for_each(move |msg| {
        println!("TCP message: {:?}", msg);
        sender.start_send(Message::arbitrary("hi back".into()));
        Ok(())
    });

    current_thread::spawn(plumb!("tcp.read", conn));
    Ok(())
}

type ErrorFn = Fn(io::Error);
fn error(err: io::Error) {
    println!("server error {:?}", err);
}
