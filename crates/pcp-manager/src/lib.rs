//! The port mapper engine.

// use std::{
//     collections::HashMap,
//     net::{Ipv4Addr, SocketAddr, SocketAddrV4},
//     time::Duration,
// };

// /// The runtime parameters required for creating a port mapper loop.
// pub struct Params<Key> {
//     /// The incoming command.
//     ///
//     /// Closing will trigger the loop shutdown.
//     pub rx: tokio::sync::mpsc::Receiver<Command<Key>>,
//     /// The initial nonce.
//     pub nonce: u32,
//     /// The gateway to run the port forwarding with.
//     pub gateway: SocketAddr,
// }

// /// The commands that can be given to the port mapper loop.
// pub enum Command<Key> {
//     /// Register a port mapping.
//     Register(RegisterCommand<Key>),
//     /// Unregister a port mapping.
//     Unregister(UnregisterCommand<Key>),
//     /// Unregister all port mappings.
//     UnregisterAll(UnregisterAllCommand),
// }

// /// The request/response command shape.
// pub struct RequestResponse<Request, Response> {
//     /// The tx where to send the response when the command is completed.
//     pub response_tx: tokio::sync::oneshot::Sender<Response>,
//     /// The request data.
//     pub request: Request,
// }

// /// The request of the reigstration command.
// pub struct RegisterRequest<Key> {
//     /// The key to register this mapping as.
//     pub key: Key,
//     /// The local address to forward to.
//     pub local: SocketAddr,
//     /// The external port to from.
//     pub external_port: u16,
// }

// /// The response from the registration command.
// pub struct RegisterResponse {
//     /// The external address that the data will be forwarded from.
//     pub external: SocketAddr,
//     /// The local address that the data will be forwarded to.
//     pub local: SocketAddr,
//     /// The gateway that is doing the forwarding for us.
//     pub gateway: SocketAddr,
//     /// The lifetime of this port forward allocation.
//     pub lifetime: Duration,
// }

// /// Register a port mapping.
// pub type RegisterCommand<Key> = RequestResponse<RegisterRequest<Key>, RegisterResponse>;

// /// Unregister a port mapping.
// pub type UnregisterCommand<Key> = RequestResponse<Key, ()>;

// /// Unregister all port mappings.
// pub type UnregisterAllCommand = RequestResponse<(), ()>;

// struct Entry {
//     local: SocketAddr,
//     external: SocketAddr,
//     gateway: SocketAddr,
// }

// pub async fn run<Key: Eq + std::hash::Hash>(params: Params<Key>) {
//     let Params {
//         mut rx,
//         nonce,
//         gateway,
//     } = params;

//     let mut map: HashMap<Key, Entry> = HashMap::new();

//     while let Some(command) = rx.recv().await {
//         match command {
//             Command::Register(RequestResponse {
//                 response_tx,
//                 request:
//                     RegisterRequest {
//                         key,
//                         local,
//                         external_port,
//                     },
//             }) => match map.entry(key) {
//                 std::collections::hash_map::Entry::Occupied(mut existing) => {
//                     let stored = existing.get_mut();
//                     if stored.local == local && stored.external.port() == external_port {
//                         // renew
//                     } else {
//                     }
//                 }
//                 std::collections::hash_map::Entry::Vacant(vacant) => {
//                     let external =
//                         SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), external_port).into();
//                     let lifetime = Duration::from_secs(60 * 60 * 2);
//                     vacant.insert(Entry {
//                         local,
//                         gateway,
//                         external,
//                     });

//                     response_tx.send(RegisterResponse {
//                         external,
//                         local,
//                         gateway,
//                         lifetime,
//                     });
//                 }
//             },
//         }
//     }
// }

// pub struct Handle {}

// impl app_core::PortManager for Handle {
//     type Key = String;
//     type KeyRef = str;

//     type RegistrationRequest<Key> = RegisterRequest<Key>;
//     type RegistrationResponse = RegisterResponse;

//     type RegisterError = anyhow::Error;
//     type UnregisterError = anyhow::Error;

//     async fn register(
//         &self,
//         request: Self::RegistrationRequest<Self::Key>,
//     ) -> Result<Self::RegistrationResponse, Self::RegisterError> {
//     }

//     fn unregister(&self, key: &Self::KeyRef) -> Result<(), Self::UnregisterError> {
//         todo!()
//     }
// }
