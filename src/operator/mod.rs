use constants::BIND;
use futures::{Future, MapErr, Stream};
use futures::stream::{ForEach};
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::executor::current_thread;
use tokio::net::{Incoming, TcpListener, TcpStream};
use tokio_io::{io as tio, AsyncRead};

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
    // Split up the read and write halves
    let (reader, writer) = tcp.split();

    println!("Got a connection");

    // Copy the data back to the client
    let conn = tio::copy(reader, writer)
        // print what happened
        .map(|(n, _, _)| {
            println!("wrote {} bytes", n)
        })
        // Handle any errors
        .map_err(|err| {
            println!("IO error {:?}", err)
        });

    // Spawn the future as a concurrent task
    current_thread::spawn(conn);

    Ok(())
}

type ErrorFn = Fn(io::Error);
fn error(err: io::Error) {
    println!("server error {:?}", err);
}
