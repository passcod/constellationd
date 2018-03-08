# Constellationd

[![Support this on Patreon!](https://c5.patreon.com/external/logo/become_a_patron_button.png)](https://www.patreon.com/passcod)

Constellationd is a small agent that provides simple primitives to monitor and manage _constellations_ or clusters or just non-descript groups of networked machines.

While the internals use the constellation and stars metaphor, in this documentation I've used more common terms for clarity and approachability.


## Build, install, deploy

There are several options:

- Download a prebuilt binary (not available before release)
- Use `cargo install constellationd` (not available before release)
- Clone this repo and build it yourself.

The last option also lets you include the configuration file into the executable itself, by writing it to `src/config.json`. That may make deploys easier, at the cost of not being able to alter the configuration without a rebuild (or a hex editor).

To generate the configuration, run `constellationd --keygen`. That will write out a file named `constellationd.json`. That file needs to be in the working directory when running the executable.

The configuration file is shared between every agent in the constellationd cluster. Agents that do not share a config will not be able to communicate nor even see each other.

To deploy, simply copy the binary and its configuration if necessary to the target server and hook it into your usual init processes.

### Requirements

- UDP connectivity between agents, at minimum. The agent binds to port 6776 both for TCP and for UDP. UDP is required, but TCP is recommended.
- UDP Multicast connectivity between servers. Usually that means they all need to be on the same network. This requirement may go away in the future.

### Footprint

The agent targets less than 10MB operating memory, and less than 5MB binary size. There is no persistent storage beyond writing to temp, which should be on the OS ramdisk facility.

Each agent generates less than 1kbit/s of outgoing network, and minimal CPU usage, when idle. When under load, which by design should be rare, it may use a lot more.

It is recommended to adjust nice and ionice values for large deploys.


## Operation

A cluster can be in three states:

- Idle
- Call
- Order

Most of the time, a cluster will be Idle. Call state is a transitionary state needed to reach Order state. In the Order state, the cluster responds and reports to an operator, which may be a human issuing commands or an automated client.

### Calling for Order

Many other agents or orchestrators maintain order by either answering to a central server, or by maintaining distributed consensus, or by a mixture of the two. Constellationd takes another approach: agents maintain loose awareness of their neighbours, and only obtain consensus or readiness when needed.

Connectivity between Calls need not be maintained, and Calls that don't reach all nodes are perfectly fine! The state of the cluster as of the current Call is reported to the caller, who may then make decisions based on that.

When an operator wants to know issue a command to the cluster, it issues a Call for Order with a randomly-generated Order ID to any one agent. That agent asks all other agents it can reach, and _they_ ask all other _they_ can reach, and so on, until Order is achieved.

If the cluster is already in Order, the Call fails.

If there is another operator trying to obtain Order at the same time, either one or both Calls may fail. Each operator should wait a random amount of time and try again.

Once a Call for Order succeeds, the entire cluster locks itself (until Order timeout) and only recognises queries from the current operator. Queries and responses are relayed through the cluster to and from the operator.

When the operator is done, or when the Order expires, the cluster reverts back to Idle state.

### User experience

While the above may seem complex, this process should mostly be hidden from end-users' view. Constellationd is a primitive: tools should be built upon it.

## Appendices

### TODOs

Things that are meant to be, but not implemented in this release.

- If a Hello is received on a TCP connection, enable gossip mode for the connection. Hook it up to the main ping timer somehow. That should provide at least rudimentary connectivity beyond UDP.

### Transport Envelope

Simple design, for fast prototype. But also, can be read from a stream. Cannot be written as a stream, though.

- Byte 0: version number as u8. Should be 0.
- Byte 1: length of header as u8.
- Bytes 2 to (header length + 2): header, CBOR-encoded.

The header decodes to an array with, in order:
- cluster key (can be variable)
- nonce (should be 32 bytes)
- length of payload

Entire header cannot be longer than 256 bytes, so with nonce+length+overhead and some future-proofing, the cluster key is limited to 32 bytes for now.

The rest of the data up to the payload length is an encrypted block of data, using sodium. The cluster secret and the nonce is used to decrypt it. The resulting plaintext is a CBOR-encoded structure.

This envelope can be applied both to datagrams (UDP transport) and to larger streams (TCP transport), but due to ahead-of-time encrypting of the entire payload, cannot yet be used for large TCP payloads.
