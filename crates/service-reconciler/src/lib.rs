//! The Kubernetes controller.

use std::sync::Arc;

use app_core::PortManager;
use kube::ResourceExt;

mod metrics;

use self::metrics::Metrics;

/// The type configuration used for this reconciler.
pub trait Config: std::fmt::Debug {
    /// The port manager to work with.
    type PortManager: app_core::PortManager<
            Key = String,
            KeyRef = str,
            RegistrationRequest<String> = PortForwardMap,
            RegistrationResponse = String,
            RegisterError = Self::RegisterError,
            UnregisterError = Self::UnregisterError,
        > + std::fmt::Debug;

    /// The type of an error that can occur at registration.
    type RegisterError: std::error::Error;

    /// The type of an error that can occur at unregistration.
    type UnregisterError: std::error::Error;
}

/// The port forward map used to register with the [`app_core::PortManager`].
pub struct PortForwardMap {
    /// The key type that identifies this group of forwarded ports.
    pub key: String,
    /// The ports beign forwarded.
    pub ports: Vec<PortForwardItem>,
}

/// A single port forward map item.
pub struct PortForwardItem {
    /// Route from this port.
    pub from: i32,
    /// Route to this port.
    pub to: i32,
    /// The protocol name to forward.
    pub protocol: String,
}

/// Context for the reconciler.
#[derive(Clone)]
pub struct Context<T: Config> {
    /// Kubernetes API client.
    pub client: kube::Client,
    /// Prometheus metrics.
    pub metrics: Metrics,
    /// The port manager interface.
    pub port_manager: Arc<T::PortManager>,
}

/// Errors that can occur during the reconciliation.
#[derive(derivative::Derivative, thiserror::Error)]
#[derivative(Debug)]
pub enum ReconcileError<T: Config + 'static> {
    /// The registration with the port manager failed.
    #[error("Register Error: {0}")]
    Register(#[source] <T::PortManager as app_core::PortManager>::RegisterError),

    /// The unregistration from the port manager failed.
    #[error("Unregister Error: {0}")]
    Unregister(#[source] <T::PortManager as app_core::PortManager>::UnregisterError),
}

/// The error type for this reconciler.
pub type Error<T> = kube::runtime::finalizer::Error<ReconcileError<T>>;

/// Determine the error kind and return the metric label.
fn error_metric_label<T: Config>(err: &Error<T>) -> &'static str {
    match err {
        Error::ApplyFailed(_) => "apply_failed",
        Error::CleanupFailed(_) => "cleanup_failed",
        Error::AddFinalizer(_) => "add_finalizer",
        Error::RemoveFinalizer(_) => "remove_finalizer",
        Error::UnnamedObject => "unnamed_object",
    }
}

/// How to treat the errors.
pub fn error_policy<T: Config>(
    service: Arc<k8s_openapi::api::core::v1::Service>,
    error: &Error<T>,
    ctx: Arc<Context<T>>,
) -> kube::runtime::controller::Action {
    let label = error_metric_label(error);
    tracing::warn!(message = "reconcile failed", ?error, %label);
    ctx.metrics.reconcile_failure(&service, label);
    kube::runtime::controller::Action::requeue(std::time::Duration::from_secs(5 * 60))
}

/// The finalizer string to put in the service resource.
pub static SERVICE_FINALIZER: &str = "port-forward-operator.mzg.io";

/// Perform reconciliation.
pub async fn reconcile<T: Config>(
    service: Arc<k8s_openapi::api::core::v1::Service>,
    ctx: Arc<Context<T>>,
) -> Result<kube::runtime::controller::Action, Error<T>> {
    let ns = service.namespace().unwrap(); // service is namespace scoped
    let services: kube::Api<k8s_openapi::api::core::v1::Service> =
        kube::Api::namespaced(ctx.client.clone(), &ns);

    tracing::info!(
        message = "Reconciling Service",
        name = %service.name_any(),
        namespace = %ns
    );
    kube::runtime::finalizer(&services, SERVICE_FINALIZER, service, move |event| {
        reconcile_event(event, ctx)
    })
    .await
}

/// Handle an reconciliation event.
async fn reconcile_event<T: Config>(
    event: kube::runtime::finalizer::Event<k8s_openapi::api::core::v1::Service>,
    ctx: Arc<Context<T>>,
) -> Result<kube::runtime::controller::Action, ReconcileError<T>> {
    match event {
        kube::runtime::finalizer::Event::Apply(service) => {
            let Some(request) = make_port_forward_map(service) else {
                return Ok(kube::runtime::controller::Action::requeue(
                    std::time::Duration::from_secs(10),
                ));
            };

            let _response: String = ctx
                .port_manager
                .register(request)
                .await
                .map_err(ReconcileError::Register)?;
            Ok(kube::runtime::controller::Action::requeue(
                std::time::Duration::from_secs(10),
            ))
        }
        kube::runtime::finalizer::Event::Cleanup(service) => {
            let Some(ref key) = service.metadata.uid else {
                return Ok(kube::runtime::controller::Action::requeue(
                    std::time::Duration::from_secs(10),
                ));
            };

            ctx.port_manager
                .unregister(key)
                .await
                .map_err(ReconcileError::Unregister)?;
            Ok(kube::runtime::controller::Action::await_change())
        }
    }
}

/// Make a port forward map for a given service.
fn make_port_forward_map(
    service: Arc<k8s_openapi::api::core::v1::Service>,
) -> Option<PortForwardMap> {
    let key = service.metadata.uid.as_ref()?;
    let spec = service.spec.as_ref()?;
    let ports = spec.ports.as_ref()?;

    let ports = ports
        .iter()
        .flat_map(|item| {
            let node_port = item.node_port?;
            let protocol = item.protocol.as_ref()?;

            Some(PortForwardItem {
                from: item.port,
                to: node_port,
                protocol: protocol.to_owned(),
            })
        })
        .collect();

    Some(PortForwardMap {
        key: key.to_owned(),
        ports,
    })
}
