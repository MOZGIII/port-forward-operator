//! Send operation.

use std::net::SocketAddrV4;

use pcp_proto as protocol;

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error(transparent)]
    SendTo(std::io::Error),
    #[error("unable to write whole packet: packet size {expected} but only {actual} written")]
    SizeMismatch { expected: usize, actual: usize },
}

pub async fn send(
    socket: &tokio::net::UdpSocket,
    gateway: SocketAddrV4,
    req: &protocol::Request,
) -> Result<(), SendError> {
    let encoded = req.encode();

    let sent = socket
        .send_to(&encoded, gateway)
        .await
        .map_err(SendError::SendTo)?;

    if sent != encoded.len() {
        return Err(SendError::SizeMismatch {
            expected: encoded.len(),
            actual: sent,
        });
    }

    Ok(())
}
