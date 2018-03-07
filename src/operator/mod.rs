use constants::BIND;
use futures::{Future, IntoFuture, MapErr, Stream};
use futures::stream::{ForEach};
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
    let (writer, reader) = tcp.framed(DatastreamCodec::default()).split();

    println!("Got a connection");
    let conn = reader.filter_map(|res| match res {
        Ok(m) => Some(m),
        Err(e) => {
            println!("Bad message: {:?}", e);
            None
        }
    }).for_each(|msg| {
        println!("TCP message: {:?}", msg);
        Ok(())
    }).into_future();

    // Copy the data back to the client
    // let conn = tio::copy(reader, writer)
    //     // print what happened
    //     .map(|(n, _, _)| {
    //         println!("wrote {} bytes", n)
    //     })
    //     // Handle any errors
    //     .map_err(|err| {
    //         println!("IO error {:?}", err)
    //     });

    // Spawn the future as a concurrent task
    current_thread::spawn(plumb!("tcp.conn", conn));

    Ok(())
}

type ErrorFn = Fn(io::Error);
fn error(err: io::Error) {
    println!("server error {:?}", err);
}
