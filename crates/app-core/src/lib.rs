//! Core interfaces for component interoperation.
//!
//! [`app_core`] instead of [`core`] becasue the latter is taken by the Rust itself.

use std::borrow::Borrow;

/// The port manager interface.
///
/// Provides the port mapping / forwarding mechanisms.
pub trait PortManager {
    /// The key to use for this registration.
    type Key;

    /// The ref to the key.
    type KeyRef: Borrow<Self::Key> + ?Sized;

    /// The registration request type.
    type RegistrationRequest<Key>;
    /// The registration response type.
    type RegistrationResponse;

    /// An error that can possibly occur while doing the registration.
    type RegisterError;
    /// An error that can possibly occur while doing the unregistration.
    type UnregisterError;

    /// Register a new port forward.
    ///
    /// Idempotent for the same request.
    fn register(
        &self,
        request: Self::RegistrationRequest<Self::Key>,
    ) -> impl std::future::Future<Output = Result<Self::RegistrationResponse, Self::RegisterError>> + Send;

    /// Unregister an existing port forward.
    ///
    /// Idempotent for the same key.
    fn unregister(
        &self,
        key: &Self::KeyRef,
    ) -> impl std::future::Future<Output = Result<(), Self::UnregisterError>> + Send;
}
