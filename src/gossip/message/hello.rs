use interfaces::Interface;
use itertools::Itertools;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Hello {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    interfaces: Vec<(String, u32)>,
}

impl Default for Hello {
    fn default() -> Self {
        let ifaces = Interface::get_all().unwrap_or(vec![]);
        let ifaces = ifaces.iter()
            .filter(|i| i.is_up() && !i.is_loopback())
            .flat_map(|i|
                i.hardware_addr().map(|m| m.to_string())
                .and_then(|mac|
                    i.get_mtu().map(|mtu| (mac, mtu))
                ).ok()
            )
            .filter(|&(ref a, _)| a != "00:00:00:00:00:00")
            .unique()
            .collect();

        Self {
            interfaces: ifaces,
        }
    }
}
