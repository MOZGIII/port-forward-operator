//! The port mapper based on the PCP protocol.

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::atomic::AtomicU32,
    time::Duration,
};

pub struct PcpMapper {
    pub socket: tokio::net::UdpSocket,
    pub local_ip: Ipv4Addr,
    pub gateway: SocketAddrV4,
    pub nonce: AtomicU32,
}

impl PcpMapper {
    pub fn next_nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        let nonce_num = self
            .nonce
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        nonce[0..4].copy_from_slice(nonce_num.to_ne_bytes().as_slice());
        nonce
    }
}

impl app_core::PortManager for PcpMapper {
    type RegistrationRequest<'a> = &'a Request;
    type RegistrationResponse = Response;

    type UnregistrationRequest<'a> = &'a Request;
    type UnregistrationResponse = ();

    type RegisterError = pcp_client::RegisterError;
    type UnregisterError = pcp_client::SendError;

    async fn register(
        &self,
        req: Self::RegistrationRequest<'_>,
    ) -> Result<Self::RegistrationResponse, Self::RegisterError> {
        let nonce = self.next_nonce();
        let res = pcp_client::register(
            &self.socket,
            self.gateway,
            &pcp_client::MappingRequest {
                protocol: req.protocol,
                local_ip: self.local_ip,
                local_port: req.local_port,
                requested_address: None,
                requested_port: Some(req.external_port.try_into().unwrap()),
                requested_lifetime_seconds: pcp_client::RECOMMENDED_LIFETIME_SECONDS,
                nonce,
            },
        )
        .await?;

        Ok(Response {
            local: SocketAddrV4::new(res.local_ip, res.local_port),
            external: SocketAddrV4::new(res.external_address, res.external_port.into()),
            duration: Duration::from_secs(res.lifetime_seconds.into()),
        })
    }

    async fn unregister(
        &self,
        req: Self::UnregistrationRequest<'_>,
    ) -> Result<(), Self::UnregisterError> {
        let nonce = self.next_nonce();
        pcp_client::release(
            &self.socket,
            self.gateway,
            req.protocol,
            self.local_ip,
            req.local_port,
            nonce,
        )
        .await?;
        Ok(())
    }
}

pub struct Request {
    pub local_port: u16,
    pub external_port: u16,
    pub protocol: u8,
}

pub struct Response {
    pub local: SocketAddrV4,
    pub external: SocketAddrV4,
    pub duration: Duration,
}
