//! Metrics for the controller.

use k8s_openapi::api::core::v1::Service;
use kube::ResourceExt;
use prometheus::{histogram_opts, opts, HistogramVec, IntCounter, IntCounterVec, Registry};

/// The metrics to report.
#[derive(Clone)]
pub struct Metrics {
    /// The number of reconsilications so far.
    pub reconciliations: IntCounter,
    /// The number of encountered failures.
    pub failures: IntCounterVec,
    /// The duration of a single reconciliation.
    pub reconcile_duration: HistogramVec,
}

impl Default for Metrics {
    fn default() -> Self {
        let reconcile_duration = HistogramVec::new(
            histogram_opts!(
                "service_controller_reconcile_duration_seconds",
                "The duration of reconcile to complete in seconds"
            )
            .buckets(vec![0.01, 0.1, 0.25, 0.5, 1., 5., 15., 60.]),
            &[],
        )
        .unwrap();
        let failures = IntCounterVec::new(
            opts!(
                "service_controller_reconciliation_errors_total",
                "reconciliation errors",
            ),
            &["instance", "error"],
        )
        .unwrap();
        let reconciliations = IntCounter::new(
            "service_controller_reconciliations_total",
            "reconciliations",
        )
        .unwrap();
        Metrics {
            reconciliations,
            failures,
            reconcile_duration,
        }
    }
}

impl Metrics {
    /// Register API metrics to start tracking them.
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.reconcile_duration.clone()))?;
        registry.register(Box::new(self.failures.clone()))?;
        registry.register(Box::new(self.reconciliations.clone()))?;
        Ok(self)
    }

    /// Reconrd an reconciliation failure.
    pub fn reconcile_failure(&self, service: &Service, label: &'static str) {
        self.failures
            .with_label_values(&[service.name_any().as_ref(), label])
            .inc()
    }

    /// Record the reconciliation.
    pub fn count_and_measure(&self) -> ReconcileMeasurer {
        self.reconciliations.inc();
        ReconcileMeasurer {
            start: std::time::Instant::now(),
            metric: self.reconcile_duration.clone(),
        }
    }
}

/// Smart function duration measurer
///
/// Relies on Drop to calculate duration and register the observation in the histogram
pub struct ReconcileMeasurer {
    /// The instant at which the reconciliation has begun.
    start: std::time::Instant,
    /// The metric to submit the measurement to.
    metric: HistogramVec,
}

impl Drop for ReconcileMeasurer {
    fn drop(&mut self) {
        #[allow(clippy::cast_precision_loss)]
        let duration = self.start.elapsed().as_millis() as f64 / 1000.0;
        self.metric.with_label_values(&[]).observe(duration);
    }
}
