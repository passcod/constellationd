


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

This envelope can be applied both to datagrams (UDP transport) and to larger streams (TCP transport), but due to ahead-of-time encrypting of the entire payload, cannot yet be used for large TCP payloads. Therefore envelopes with payload lengths > 60KiB will be silently discarded for now.
