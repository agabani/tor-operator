use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

use crate::{kubernetes::Resource, Error};

#[derive(Clone)]
pub struct Metrics {}

impl Metrics {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn handle(&self) -> PrometheusHandle {
        const EXPONENTIAL_SECONDS: &[f64] = &[0.01, 0.1, 0.25, 0.5, 1.0, 5.0, 15.0, 60.0];

        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("tor_operator_reconcile_duration_seconds".to_string()),
                EXPONENTIAL_SECONDS,
            )
            .unwrap()
            .install_recorder()
            .unwrap()
    }

    #[must_use]
    pub fn count_and_measure(&self, controller: &'static str) -> ControllerTimer {
        metrics::increment_counter!(
            "tor_operator_reconciliations_total",
            "controller" => controller
        );
        ControllerTimer {
            start: std::time::Instant::now(),
            metric: "tor_operator_reconcile_duration_seconds",
            controller,
        }
    }

    pub fn reconcile_failure(&self, controller: &'static str, error: &Error) {
        let error = match error {
            Error::Kube(_) => "kube",
            Error::MissingObjectKey(_) => "missing object key",
        };
        metrics::increment_counter!(
            "tor_operator_reconciliation_errors_total",
            "controller" => controller,
            "error" => error
        );
    }

    pub fn kubernetes_api_usage_count<R>(verb: &'static str)
    where
        R: Resource,
    {
        metrics::increment_counter!(
            "tor_operator_kubernetes_api_usage_total",
            "kind" => R::kind(&()),
            "group" => R::group(&()),
            "verb" => verb,
            "version" => R::version(&()),
        );
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ControllerTimer {
    start: std::time::Instant,
    metric: &'static str,
    controller: &'static str,
}

impl Drop for ControllerTimer {
    fn drop(&mut self) {
        metrics::histogram!(
            self.metric.to_string(),
            self.start.elapsed().as_secs_f64(),
            &[("controller", self.controller)]
        );
    }
}
