//! Definitions and utilities to interact with a PCP server.

// Borrowed from <https://github.com/n0-computer/iroh/tree/main/iroh-net/src/portmapper>.

use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4},
    num::{NonZeroU16, NonZeroU32},
    time::Duration,
};

use pcp_proto as protocol;
use tracing::{debug, trace};

mod recv;
mod send;
mod validate;

pub use self::recv::*;
pub use self::send::*;
pub use self::validate::*;

// PCP and NAT-PMP share same ports, reassigned by IANA from the older version to the new one. See
// <https://datatracker.ietf.org/doc/html/rfc6887#section-19>

/// Port to use when acting as a server. This is the one we direct requests to.
pub const SERVER_PORT: u16 = 5351;

/// Timeout to receive a response from a PCP server.
const RECV_TIMEOUT: Duration = Duration::from_millis(500);

/// Expose the recommended port mapping lifetime for PMP, which is 2 hours. See
/// <https://datatracker.ietf.org/doc/html/rfc6886#section-3.3>
#[allow(unsafe_code)]
pub const RECOMMENDED_LIFETIME_SECONDS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(60 * 60) };

/// A mapping sucessfully registered with a PCP server.
#[derive(Debug)]
pub struct Mapping {
    /// Local ip used to create this mapping.
    pub local_ip: Ipv4Addr,
    /// Local port used to create this mapping.
    pub local_port: NonZeroU16,
    /// External port of the mapping.
    pub external_port: NonZeroU16,
    /// External address of the mapping.
    pub external_ip: Ipv4Addr,
    /// Allowed time for this mapping as informed by the server.
    pub lifetime_seconds: u32,
    /// The nonce of the mapping, used for modifications with the PCP server, for example releasing
    /// the mapping.
    pub nonce: [u8; 12],
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("send: {0}")]
    Send(#[source] SendError),
    #[error("recv: {0}")]
    Recv(#[source] RecvError),
    #[error("validation: {0}")]
    Validate(#[source] ValidateError),
    #[error("unexpected opcode: {0}")]
    UnexpectedOpcode(#[source] ValidateError),
}

/// Send a request and read a response.
pub async fn request(
    socket: &tokio::net::UdpSocket,
    gateway: SocketAddrV4,
    req: &protocol::Request,
) -> Result<protocol::Response, RequestError> {
    send(socket, gateway, req)
        .await
        .map_err(RequestError::Send)?;

    let mut buffer = [0; protocol::Response::MAX_SIZE];
    let res = recv(socket, &mut buffer, RECV_TIMEOUT)
        .await
        .map_err(RequestError::Recv)?;

    validate(req, &res).map_err(RequestError::Validate)?;

    Ok(res)
}

#[derive(Debug)]
pub struct MappingRequest {
    pub protocol: u8,
    pub local_ip: Ipv4Addr,
    pub local_port: u16,
    pub requested_address: Option<Ipv4Addr>,
    pub requested_port: Option<NonZeroU16>,
    pub requested_lifetime_seconds: NonZeroU32,
    pub nonce: [u8; 12],
}

#[derive(Debug)]
pub struct MappingResponse {
    pub protocol: u8,
    pub local_ip: Ipv4Addr,
    pub local_port: u16,
    pub external_address: Ipv4Addr,
    pub external_port: NonZeroU16,
    pub lifetime_seconds: u32,
    pub nonce: [u8; 12],
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("request: {0}")]
    Request(RequestError),
    #[error("the external IP is not v4: {0}")]
    ExternalIpIsNotV4(Ipv6Addr),
    #[error("the external port is zero: {0}")]
    ExternalPortIsZero(std::num::TryFromIntError),
}

/// Attempt to register a new mapping with the PCP server on the provided gateway.
pub async fn register(
    socket: &tokio::net::UdpSocket,
    gateway: SocketAddrV4,
    req: &MappingRequest,
) -> Result<MappingResponse, RegisterError> {
    let proto_req = protocol::Request::mapping(
        req.nonce,
        req.protocol,
        req.local_port,
        req.local_ip,
        req.requested_port.map(Into::into),
        req.requested_address,
        req.requested_lifetime_seconds.into(),
    );

    let proto_res = request(socket, gateway, &proto_req)
        .await
        .map_err(RegisterError::Request)?;

    let protocol::OpcodeData::MapData(map) = proto_res.data else {
        unreachable!();
    };

    let external_address = map
        .external_address
        .to_ipv4_mapped()
        .ok_or_else(|| RegisterError::ExternalIpIsNotV4(map.external_address))?;

    let external_port = map
        .external_port
        .try_into()
        .map_err(RegisterError::ExternalPortIsZero)?;

    Ok(MappingResponse {
        protocol: map.protocol,
        local_ip: req.local_ip,
        local_port: map.local_port,
        external_address,
        external_port,
        lifetime_seconds: proto_res.lifetime_seconds,
        nonce: map.nonce,
    })
}

/// Attempt to release a (preexising) mapping at the PCP server on the provided gateway.
pub async fn release(
    socket: &tokio::net::UdpSocket,
    gateway: SocketAddrV4,
    protocol: u8,
    local_ip: Ipv4Addr,
    local_port: u16,
    nonce: [u8; 12],
) -> Result<(), SendError> {
    let req = protocol::Request::mapping(nonce, protocol, local_port, local_ip, None, None, 0);
    send(socket, gateway, &req).await?;
    // mapping deletion is a notification, no point in waiting for the response
    Ok(())
}

/// Send the probe request to the local gateway and reads the response.
async fn probe(
    socket: &tokio::net::UdpSocket,
    gateway: SocketAddrV4,
    local_ip: Ipv4Addr,
) -> anyhow::Result<protocol::Response> {
    let req = protocol::Request::annouce(local_ip.to_ipv6_mapped());
    let res = request(socket, gateway, &req).await?;
    Ok(res)
}

/// Probes the local gateway for PCP support.
pub async fn is_gateway_available(
    socket: &tokio::net::UdpSocket,
    gateway: SocketAddrV4,
    local_ip: Ipv4Addr,
) -> bool {
    match probe(socket, gateway, local_ip).await {
        Ok(response) => {
            trace!("probe response: {response:?}");
            let protocol::Response {
                lifetime_seconds: _,
                epoch_time: _,
                data,
            } = response;
            match data {
                protocol::OpcodeData::Announce => true,
                _ => {
                    debug!("server returned an unexpected response type for probe");
                    // missbehaving server is not useful
                    false
                }
            }
        }
        Err(e) => {
            debug!("probe failed: {e}");
            false
        }
    }
}
