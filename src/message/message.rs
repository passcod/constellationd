use constants;
use statics;
use super::hello::Hello;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Body {
    Arbitrary(String),
    Hello(Hello),
    Other,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Message {
    pub agent: (String, String),
    pub id: String,
    pub body: Body,
}

macro_rules! variant {
    ($low:ident, $high:ident) => (
        pub fn $low() -> Self {
            Self::new(Body::$high(
                $high::default()
            ))
        }
    )
}

impl Message {
    pub fn new(body: Body) -> Self {
        Message {
            id: statics::id().clone(),
            agent: (
                constants::AGENT.0.into(),
                constants::AGENT.1.into()
            ),
            body: body,
        }
    }

    pub fn arbitrary(msg: String) -> Self {
        Self::new(Body::Arbitrary(msg))
    }

    variant!(hello, Hello);
}
