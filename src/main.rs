extern crate net2;
extern crate base64;
extern crate rust_sodium;
extern crate futures;
extern crate tokio_core;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use futures::{Sink, Stream};
use net2::UdpBuilder;
use net2::unix::UnixUdpBuilderExt;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::{Core, Handle};

const PROTOCOL_VERSION: u8 = 0;
const BIND: &'static str = "0.0.0.0:6776";
const CAST: &'static str = "224.0.247.51:6776";
const MULTI: [u8; 4] = [224, 0, 247, 51];
const ANY: [u8; 4] = [0, 0, 0, 0];

fn udp(handle: &Handle) -> io::Result<UdpSocket> {
    let sock = UdpBuilder::new_v4()?
        .reuse_address(true)?
        .reuse_port(true)?
        .bind(BIND)?;

    sock.set_broadcast(true)?;
    sock.set_multicast_loop_v4(true)?;
    sock.set_multicast_ttl_v4(1)?; // Set higher to reach outside local
    sock.join_multicast_v4(&MULTI.into(), &ANY.into())?;

    // test message buffered by kernel and received immediately
    // by ourselves so we can check the tokio stack works
    let msg = serde_json::to_vec(&Message::new(None)).unwrap();
    sock.send_to(
        &msg,
        &SocketAddr::from_str(CAST).unwrap()
    ).expect("Failed to send");

    UdpSocket::from_socket(sock, handle)
}

fn id() -> &'static String {
    lazy_static! {
        static ref ID: String = {
            let bytes = rust_sodium::randombytes::randombytes(16);
            base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
        };
    }

    &ID
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
struct Message {
    v: u8,
    agent: (String, String),
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
}

impl Message {
    fn new(body: Option<String>) -> Self {
        Message {
            v: PROTOCOL_VERSION,
            id: id().clone(),
            agent: (
                env!("CARGO_PKG_NAME").into(),
                env!("CARGO_PKG_VERSION").into()
            ),
            body: body,
        }
    }
}

struct JsonCodec;

impl UdpCodec for JsonCodec {
    type In = Option<Message>;
    type Out = Message;

    fn decode(&mut self, _: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        serde_json::from_slice(buf)
            .or_else(|err| {
                println!("Bad message: {:?}\n{:?}", buf, err);
                Ok(None)
            })
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        let ser = serde_json::to_vec(&msg).expect("Unable to encode message");
        buf.extend(ser);
        SocketAddr::from_str(CAST).unwrap()
    }
}

fn main() {
    println!("{} v{}\nID: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        id()
    );

    if !rust_sodium::init() {
        panic!("Failed to initialise sodium");
    }

    let mut core = Core::new().expect("Failed to initialise event loop");
    let handle = core.handle();

    let (mut writer, reader) = udp(&handle).expect("Failed to bind UDP")
        .framed(JsonCodec).split();

    let server = reader.for_each(|msg| {
        let msg = match msg {
            None => return Ok(()),
            Some(m) => m
        };

        // Ignore own messages
        if &msg.id == id() { return Ok(()) }

        if msg.body == Some("ping".into()) {
            if let Err(err) = writer.start_send(Message::new(Some("pong".into()))) {
                println!("Failed send: {:?}", err);
            }
        }

        println!("{:?}", msg);
        Ok(())
    });

    core.run(server).expect("Failed to start UDP server");
}
