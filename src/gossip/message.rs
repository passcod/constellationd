use constants;
use statics;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Message {
    pub agent: (String, String),
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl Message {
    pub fn new(body: Option<String>) -> Self {
        Message {
            id: statics::id().clone(),
            agent: (
                constants::AGENT.0.into(),
                constants::AGENT.1.into()
            ),
            body: body,
        }
    }
}
