//! PCP opcodes and messages.

// Borrowed from <https://github.com/n0-computer/iroh/tree/main/iroh-net/src/portmapper>.

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub mod opcode_data;
pub mod request;
pub mod response;

pub use opcode_data::*;
pub use request::*;
pub use response::*;

/// NAT-PMP/PCP Version
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Version {
    /// PCP Version according to [RFC 6887 Version Negotiation](https://datatracker.ietf.org/doc/html/rfc6887#section-9)
    // Version 2
    Pcp = 2,
}

/// Opcode as defined in [RFC 6887 IANA Considerations](https://datatracker.ietf.org/doc/html/rfc6887#section-19)
// NOTE: PEER is not used, therefor not implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    /// Annouce Opcode.
    ///
    /// Used by the server to annouce changes to clients. These include restarts
    /// (indicating loss of state) and changes to mappings and external ip addresses.
    ///
    /// See [RFC 6887 ANNOUNCE Opcode](https://datatracker.ietf.org/doc/html/rfc6887#section-14.1)
    Announce = 0,
    /// Map Opcode,
    ///
    /// Used to deal with endpoint-idependent mappings.
    ///
    /// See [RFC 6887 MAP Opcode](https://datatracker.ietf.org/doc/html/rfc6887#section-11)
    Map = 1,
}
