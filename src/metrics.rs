use opentelemetry::{
    KeyValue,
    metrics::{Counter, Histogram, MeterProvider as _},
};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::Registry;

use crate::{Error, cli::CliArgs, kubernetes::Resource};

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

        let mut builder = SdkMeterProvider::builder().with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_service_name("tor-operator")
                .build(),
        );
        if let Some(opentelemetry_endpoint) = cli.opentelemetry_endpoint.as_ref() {
            let Some(opentelemetry_transport) = cli.opentelemetry_transport.as_ref() else {
                panic!("TODO: opentelemetry_transport is required")
            };

            builder = builder.with_periodic_exporter(match opentelemetry_transport.as_str() {
                "grpc" => MetricExporter::builder()
                    .with_tonic()
                    .with_endpoint(opentelemetry_endpoint)
                    .build()
                    .unwrap(),
                "http" => MetricExporter::builder()
                    .with_http()
                    .with_endpoint(format!("{}/v1/metrics", opentelemetry_endpoint))
                    .build()
                    .unwrap(),
                transport => panic!("Unsupported opentelemetry_transport: {}", transport),
            })
        } else {
            panic!("TODO: opentelemetry_endpoint is required")
        }

        builder = builder
            .with_periodic_exporter(opentelemetry_stdout::MetricExporterBuilder::default().build());

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
