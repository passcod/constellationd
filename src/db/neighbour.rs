use std::time::SystemTime;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Neighbour {
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

impl Default for Neighbour {
    fn default() -> Self {
        Self {
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        }
    }
}

impl Neighbour {
    pub fn seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
}
