use constants;
use statics;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Kind {
    Hello,
    Ping,
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
        }
    }

    pub fn hello() -> Self {
        Self::new(Kind::Hello)
    }

    pub fn ping() -> Self {
        Self::new(Kind::Ping)
    }
}
