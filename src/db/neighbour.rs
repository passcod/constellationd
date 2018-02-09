use std::time::SystemTime;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Neighbour {
    first_seen: SystemTime,
}

impl Default for Neighbour {
    fn default() -> Self {
        Self {
            first_seen: SystemTime::now(),
        }
    }
}
