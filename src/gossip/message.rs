use constants;
use statics;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Kind {
    Hello,
    Ping,
    Pong,
}

impl Kind {
    pub fn is_ping(&self) -> bool {
        self == &Kind::Ping
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Message {
    pub agent: (String, String),
    pub id: String,

    pub kind: Kind,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<usize>,
}

impl Message {
    pub fn new(kind: Kind) -> Self {
        Message {
            id: statics::id().clone(),
            agent: (
                constants::AGENT.0.into(),
                constants::AGENT.1.into()
            ),
            kind: kind,
            seq: None,
        }
    }

    pub fn hello() -> Self {
        Self::new(Kind::Hello)
    }

    pub fn pong(seq: usize) -> Self {
        let mut msg = Self::new(Kind::Pong);
        msg.seq = Some(seq);
        msg
    }
}
