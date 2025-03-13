use opentelemetry::{
    metrics::{Counter, Histogram, MeterProvider as _},
    KeyValue,
};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::Registry;

use crate::{cli::CliArgs, kubernetes::Resource, Error};

#[derive(Clone)]
pub struct Metrics {
    registry: prometheus::Registry,
    tor_operator_kubernetes_api_usage_total: Counter<u64>,
    tor_operator_reconciliation_errors_total: Counter<u64>,
    tor_operator_reconciliations_total: Counter<u64>,
    tor_operator_reconcile_duration_seconds: Histogram<f64>,
}

impl Metrics {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(cli: &CliArgs) -> Self {
        let registry = prometheus::Registry::new();

        let mut builder = SdkMeterProvider::builder()
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_attribute(KeyValue::new("service.name", "tor-operator"))
                    .with_service_name("tor-operator")
                    .build(),
            )
            .with_reader(
                opentelemetry_prometheus::exporter()
                    .with_registry(registry.clone())
                    .build()
                    .unwrap(),
            );
        if let Some(opentelemetry_endpoint) = cli.opentelemetry_endpoint.as_ref() {
            builder = builder.with_periodic_exporter(
                MetricExporter::builder()
                    .with_tonic()
                    .with_endpoint(opentelemetry_endpoint)
                    .build()
                    .unwrap(),
            );
        } else {
            panic!("opentelemetry_endpoint is required")
        }
        let provider = builder.build();

        let meter = provider.meter("tor-operator");

        let tor_operator_kubernetes_api_usage_total = meter
            .u64_counter("tor_operator_kubernetes_api_usage_total")
            .with_description("The total number of Kubernetes API requests made.")
            .build();

        let tor_operator_reconciliation_errors_total = meter
            .u64_counter("tor_operator_reconciliation_errors_total")
            .with_description("The total number of reconciliation errors.")
            .build();

        let tor_operator_reconciliations_total = meter
            .u64_counter("tor_operator_reconciliations_total")
            .with_description("The total number of reconciliations.")
            .build();

        let tor_operator_reconcile_duration_seconds = meter
            .f64_histogram("tor_operator_reconcile_duration_seconds")
            .with_description("The reconcile duration in seconds.")
            .with_unit("s")
            .build();

        opentelemetry::global::set_meter_provider(provider.clone());

        Self {
            registry,
            tor_operator_kubernetes_api_usage_total,
            tor_operator_reconciliation_errors_total,
            tor_operator_reconciliations_total,
            tor_operator_reconcile_duration_seconds,
        }
    }

    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    #[must_use]
    pub fn count_and_measure(&self, controller: &'static str) -> ControllerTimer {
        self.tor_operator_reconciliations_total
            .add(1, &[KeyValue::new("controller", controller)]);
        ControllerTimer {
            start: std::time::Instant::now(),
            metric: self.tor_operator_reconcile_duration_seconds.clone(),
            controller,
        }
    }

    pub fn reconcile_failure(&self, controller: &'static str, error: &Error) {
        let error = match error {
            Error::Kube(_) => "kube",
            Error::MissingObjectKey(_) => "missing object key",
        };
        self.tor_operator_reconciliation_errors_total.add(
            1,
            &[
                KeyValue::new("controller", controller),
                KeyValue::new("error", error),
            ],
        );
    }

    pub fn kubernetes_api_usage_count<R>(&self, verb: &'static str)
    where
        R: Resource,
    {
        self.tor_operator_kubernetes_api_usage_total.add(
            1,
            &[
                KeyValue::new("kind", R::kind(&())),
                KeyValue::new("group", R::group(&())),
                KeyValue::new("verb", verb),
                KeyValue::new("version", R::version(&())),
            ],
        );
    }
}

pub struct ControllerTimer {
    start: std::time::Instant,
    metric: Histogram<f64>,
    controller: &'static str,
}

impl Drop for ControllerTimer {
    fn drop(&mut self) {
        self.metric.record(
            self.start.elapsed().as_secs_f64(),
            &[KeyValue::new("controller", self.controller)],
        );
    }
}
