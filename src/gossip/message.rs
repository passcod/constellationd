use constants;
use statics::id;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Message {
    pub v: u8,
    pub agent: (String, String),
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl Message {
    pub fn new(body: Option<String>) -> Self {
        Message {
            v: constants::PROTOCOL_VERSION,
            id: id().clone(),
            agent: (
                env!("CARGO_PKG_NAME").into(),
                env!("CARGO_PKG_VERSION").into()
            ),
            body: body,
        }
    }
}
