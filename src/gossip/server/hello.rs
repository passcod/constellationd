use db::{self, Neighbour};
use gossip::message::{Hello, Message};

pub fn serve(msg: &Message, hello: &Hello) {
    let db = db::open::<Neighbour>();
    let mut n = if let Ok(Some(mut n)) = db.get(&msg.id) {
        n
    } else {
        Neighbour::default()
    };

    n.seen();
    n.last_hello = Some(hello.clone());

    let _ = db.set(&msg.id, &n);
    println!("Got a ping from {}!\nFirst seen: {:?}\nLast Seen: {:?}",
        msg.id, n.first_seen, n.last_seen
    );
}
