use constants::BIND;
use futures::future::{Future, ok as f_ok};
use hyper::error::{Error as HyperError, Result as HyperResult};
use hyper::header::ContentLength;
use hyper::server::{AddrIncoming, Http, Request, Response, Serve, Service};
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::reactor::Handle;

pub fn server<'a>() -> HyperResult<Serve<AddrIncoming, &'a NewOpFn>> {
    Http::new().serve_addr_handle(
        &SocketAddr::from_str(BIND).unwrap(),
        &Handle::default(),
        &new_op
    )
}

type NewOpFn = Fn() -> io::Result<Operator>;
fn new_op() -> io::Result<Operator> { Ok(Operator) }

struct Operator;

const PHRASE: &'static str = "Hello, World!";

impl Service for Operator {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = HyperError;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, _req: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        Box::new(f_ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(PHRASE)
        ))
    }
}
