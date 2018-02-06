/// The User Agent tuple (name of program, version of program).
pub const AGENT: (&'static str, &'static str) = (
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_VERSION")
);

/// The IPv4 "any address".
pub const ANY: [u8; 4] = [0, 0, 0, 0];

/// The local address+port to bind the UDP socket to.
///
/// It should always be 0.0.0.0:PORT.
pub const BIND: &'static str = "0.0.0.0:6776";

/// The address+port to multicast to.
///
/// That behaves like the "topic" for datagrams. We'll only receive messages
/// sent to the same address+port. Should be MULTI:PORT.
///
/// The IP was picked at random in the 224.0.120-249.0-255 unassigned block.
/// The port was also picked at random in the 6000 range, as regular as it seems.
pub const CAST: &'static str = "224.0.247.51:6776";

/// The name of the config file.
pub const CONFIG_FILE: &'static str = "constellationd.json";

/// The multicast address.
///
/// See CAST for a description.
pub const MULTI: [u8; 4] = [224, 0, 247, 51];

/// The version of the protocol.
pub const PROTOCOL_VERSION: u8 = 0;
