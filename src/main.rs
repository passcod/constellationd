extern crate net2;
extern crate base64;
extern crate rust_sodium;
extern crate futures;
extern crate tokio_core;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use futures::stream::Stream;
use net2::UdpBuilder;
use net2::unix::UnixUdpBuilderExt;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::{Core, Handle};

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
    let msg = serde_json::to_vec(&Message {
        v: 0,
        id: "test".into(),
        agent: (
            env!("CARGO_PKG_NAME").into(),
            env!("CARGO_PKG_VERSION").into()
        ),
        body: None,
    }).unwrap();
    sock.send_to(
        &msg,
        &SocketAddr::from_str(CAST).unwrap()
    ).expect("Failed to send");

    UdpSocket::from_socket(sock, handle)
}

fn id() -> String {
    let bytes = rust_sodium::randombytes::randombytes(16);
    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
struct Message {
    v: u8,
    agent: (String, String),
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
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
    let id = id();
    println!("{} v{}\nID: {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        id
    );

    if !rust_sodium::init() {
        panic!("Failed to initialise sodium");
    }

    let mut core = Core::new().expect("Failed to initialise event loop");
    let handle = core.handle();

    let udp = udp(&handle).expect("Failed to bind UDP").framed(JsonCodec);

    let server = udp.for_each(|msg| {
        let msg = match msg {
            None => return Ok(()),
            Some(m) => m
        };

        // Ignore own messages
        if msg.id == id { return Ok(()) }

        println!("{:?}", msg);
        // println!("addr: {:?} msg: {:?}", addr, message);
        Ok(())
    });

    core.run(server).expect("Failed to start UDP server");
}
