use constants;
use statics;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Kind {
    Hello,
    Ping,
    Pong,
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
}
