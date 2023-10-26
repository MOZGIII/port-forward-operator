//! Core interfaces for component interoperation.
//!
//! [`app_core`] instead of [`core`] becasue the latter is taken by the Rust itself.

use std::borrow::Borrow;

/// The port manager interface.
///
/// Provides the port mapping / forwarding mechanisms.
pub trait PortManager {
    /// The registration request type.
    type RegistrationRequest<'a>;
    /// The registration response type.
    type RegistrationResponse;

    /// The unregistration request type.
    type UnregistrationRequest<'a>;
    /// The unregistration response type.
    type UnregistrationResponse;

    /// An error that can possibly occur while doing the registration.
    type RegisterError;
    /// An error that can possibly occur while doing the unregistration.
    type UnregisterError;

    /// Register a new port forward.
    ///
    /// Idempotent for the same request.
    fn register(
        &self,
        request: Self::RegistrationRequest<'_>,
    ) -> impl std::future::Future<Output = Result<Self::RegistrationResponse, Self::RegisterError>> + Send;

    /// Unregister an existing port forward.
    ///
    /// Idempotent for the same request.
    fn unregister(
        &self,
        request: Self::UnregistrationRequest<'_>,
    ) -> impl std::future::Future<Output = Result<Self::UnregistrationResponse, Self::UnregisterError>>
           + Send;
}
