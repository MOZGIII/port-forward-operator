//! Recv operation.

use std::time::Duration;

use pcp_proto as protocol;

#[derive(Debug, thiserror::Error)]
pub enum RecvError {
    #[error(transparent)]
    Recv(std::io::Error),
    #[error("timeout: {0}")]
    Timeout(#[source] tokio::time::error::Elapsed),
    #[error("pcp protocol: {0}")]
    Protocol(#[source] protocol::Error),
}

pub async fn recv(
    socket: &tokio::net::UdpSocket,
    buf: &mut [u8; protocol::Response::MAX_SIZE],
    timeout: Duration,
) -> Result<protocol::Response, RecvError> {
    let read = tokio::time::timeout(timeout, socket.recv(buf))
        .await
        .map_err(RecvError::Timeout)?
        .map_err(RecvError::Recv)?;
    let response = protocol::Response::decode(&buf[..read]).map_err(RecvError::Protocol)?;
    Ok(response)
}
