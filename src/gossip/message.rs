use constants;
use interfaces;
use itertools::Itertools;
use statics;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Hello {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    mac_addresses: Vec<String>,
}

impl Default for Hello {
    fn default() -> Self {
        let ifaces = interfaces::Interface::get_all().unwrap_or(vec![]);
        let macs = ifaces.iter()
            .flat_map(|i| i
                .hardware_addr()
                .map(|m| m.to_string())
                .ok()
            )
            .filter(|m| m != "00:00:00:00:00:00")
            .unique()
            .collect();

        Self {
            mac_addresses: macs
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Body {
    Hello(Hello),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Message {
    pub agent: (String, String),
    pub id: String,
    pub body: Body,
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

    pub fn hello() -> Self { Self::new(Body::Hello(Hello::default())) }
    pub fn is_hello(&self) -> bool {
        match self.body {
            Body::Hello(_) => true,
            _ => false
        }
    }
}
