use std::{collections::HashMap, time::Duration};

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{
    Compression, Protocol, WithExportConfig, WithHttpConfig, WithTonicConfig,
    tonic_types::metadata::MetadataMap,
};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    EnvFilter, Layer as _, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

use crate::cli::{CliArgs, CliArgsOtelExporter};

pub struct Provider {
    logger: opentelemetry_sdk::logs::SdkLoggerProvider,
    meter: opentelemetry_sdk::metrics::SdkMeterProvider,
    service_name: String,
    tracer: opentelemetry_sdk::trace::SdkTracerProvider,
}

impl Provider {
    #[must_use]
    pub fn new(cli: &CliArgs) -> Self {
        Self {
            logger: logger_provider(cli),
            meter: meter_provider(cli),
            service_name: service_name(cli),
            tracer: tracer_provider(cli),
        }
    }

    #[must_use]
    pub fn meter(&self) -> &opentelemetry_sdk::metrics::SdkMeterProvider {
        &self.meter
    }

    pub fn init_tracing_subscriber(&self) {
        let logger_layer =
            OpenTelemetryTracingBridge::new(&self.logger).with_filter(external_component_filter());

        let tracer_layer = OpenTelemetryLayer::new(self.tracer.tracer(self.service_name.clone()))
            .with_filter(external_component_filter());

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_thread_names(true)
            .with_filter(EnvFilter::from_default_env());

        tracing_subscriber::registry()
            .with(logger_layer)
            .with(tracer_layer)
            .with(fmt_layer)
            .init();
    }

    /// # Errors
    ///
    /// Will return `Err` if open telemetry providers could not shutdown.
    pub fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut shutdown_errors = Vec::new();
        if let Err(e) = self.tracer.shutdown() {
            shutdown_errors.push(format!("tracer provider: {e}"));
        }
        if let Err(e) = self.meter.shutdown() {
            shutdown_errors.push(format!("meter provider: {e}"));
        }
        if let Err(e) = self.logger.shutdown() {
            shutdown_errors.push(format!("logger provider: {e}"));
        }
        if !shutdown_errors.is_empty() {
            return Err(format!(
                "Failed to shutdown providers:{}",
                shutdown_errors.join("\n")
            )
            .into());
        }

        Ok(())
    }
}

/// To prevent a telemetry-induced-telemetry loop, OpenTelemetry's own internal
/// logging is properly suppressed. However, logs emitted by external components
/// (such as reqwest, tonic, etc.) are not suppressed as they do not propagate
/// OpenTelemetry context. Until this issue is addressed
/// <https://github.com/open-telemetry/opentelemetry-rust/issues/2877>,
/// filtering like this is the best way to suppress such logs.
///
/// The filter levels are set as follows:
/// - Allow `info` level and above by default.
/// - Completely restrict logs from `hyper`, `tonic`, `h2`, and `reqwest`.
///
/// Note: This filtering will also drop logs from these components even when
/// they are used outside of the OTLP Exporter.
fn external_component_filter() -> EnvFilter {
    EnvFilter::from_default_env()
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap())
}

fn resource(cli: &CliArgs) -> opentelemetry_sdk::Resource {
    opentelemetry_sdk::Resource::builder()
        .with_service_name(service_name(cli))
        .build()
}

fn service_name(cli: &CliArgs) -> String {
    cli.otel_service_name.clone()
}

/*
 * ============================================================================
 * Compressions
 * ============================================================================
 */

fn logs_compression(cli: &CliArgs) -> Option<Compression> {
    cli.otel_exporter_otlp_logs_compression
        .or(cli.otel_exporter_otlp_compression)
        .map(Into::into)
}

fn metrics_compression(cli: &CliArgs) -> Option<Compression> {
    cli.otel_exporter_otlp_metrics_compression
        .or(cli.otel_exporter_otlp_compression)
        .map(Into::into)
}

fn traces_compression(cli: &CliArgs) -> Option<Compression> {
    cli.otel_exporter_otlp_traces_compression
        .or(cli.otel_exporter_otlp_compression)
        .map(Into::into)
}

/*
 * ============================================================================
 * Endpoints
 * ============================================================================
 */

fn logs_endpoint(cli: &CliArgs, protocol: Protocol) -> String {
    if let Some(endpoint) = &cli.otel_exporter_otlp_logs_endpoint {
        return endpoint.clone();
    }

    if let Some(endpoint) = &cli.otel_exporter_otlp_endpoint {
        return match protocol {
            Protocol::Grpc => endpoint.clone(),
            Protocol::HttpBinary | Protocol::HttpJson => {
                format!("{endpoint}/v1/logs")
            }
        };
    }

    panic!("OTEL_EXPORTER_OTLP_LOGS_ENDPOINT or OTEL_EXPORTER_OTLP_ENDPOINT must be set");
}

fn metrics_endpoint(cli: &CliArgs, protocol: Protocol) -> String {
    if let Some(endpoint) = &cli.otel_exporter_otlp_metrics_endpoint {
        return endpoint.clone();
    }

    if let Some(endpoint) = &cli.otel_exporter_otlp_endpoint {
        return match protocol {
            Protocol::Grpc => endpoint.clone(),
            Protocol::HttpBinary | Protocol::HttpJson => {
                format!("{endpoint}/v1/metrics")
            }
        };
    }

    panic!("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT or OTEL_EXPORTER_OTLP_ENDPOINT must be set");
}

fn traces_endpoint(cli: &CliArgs, protocol: Protocol) -> String {
    if let Some(endpoint) = &cli.otel_exporter_otlp_traces_endpoint {
        return endpoint.clone();
    }

    if let Some(endpoint) = &cli.otel_exporter_otlp_endpoint {
        return match protocol {
            Protocol::Grpc => endpoint.clone(),
            Protocol::HttpBinary | Protocol::HttpJson => {
                format!("{endpoint}/v1/traces")
            }
        };
    }

    panic!("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT or OTEL_EXPORTER_OTLP_ENDPOINT must be set");
}

/*
 * ============================================================================
 * Headers
 * ============================================================================
 */

fn log_headers(cli: &CliArgs) -> Option<&str> {
    cli.otel_exporter_otlp_logs_headers
        .as_ref()
        .or(cli.otel_exporter_otlp_headers.as_ref())
        .map(String::as_str)
}

fn metrics_headers(cli: &CliArgs) -> Option<&str> {
    cli.otel_exporter_otlp_metrics_headers
        .as_ref()
        .or(cli.otel_exporter_otlp_headers.as_ref())
        .map(String::as_str)
}

fn traces_headers(cli: &CliArgs) -> Option<&str> {
    cli.otel_exporter_otlp_traces_headers
        .as_ref()
        .or(cli.otel_exporter_otlp_headers.as_ref())
        .map(String::as_str)
}

/*
 * ============================================================================
 * Protocols
 * ============================================================================
 */

fn logs_protocol(cli: &CliArgs) -> Protocol {
    cli.otel_exporter_otlp_logs_protocol
        .or(cli.otel_exporter_otlp_protocol)
        .expect("OTEL_EXPORTER_OTLP_LOGS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL must be set")
        .into()
}

fn metrics_protocol(cli: &CliArgs) -> Protocol {
    cli.otel_exporter_otlp_metrics_protocol
        .or(cli.otel_exporter_otlp_protocol)
        .expect("OTEL_EXPORTER_OTLP_METRICS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL must be set")
        .into()
}

fn traces_protocol(cli: &CliArgs) -> Protocol {
    cli.otel_exporter_otlp_traces_protocol
        .or(cli.otel_exporter_otlp_protocol)
        .expect("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL must be set")
        .into()
}

/*
 * ============================================================================
 * Timeouts
 * ============================================================================
 */

fn logs_timeout(cli: &CliArgs) -> u64 {
    cli.otel_exporter_otlp_logs_timeout
        .unwrap_or(cli.otel_exporter_otlp_timeout)
}

fn metrics_timeout(cli: &CliArgs) -> u64 {
    cli.otel_exporter_otlp_metrics_timeout
        .unwrap_or(cli.otel_exporter_otlp_timeout)
}

fn traces_timeout(cli: &CliArgs) -> u64 {
    cli.otel_exporter_otlp_traces_timeout
        .unwrap_or(cli.otel_exporter_otlp_timeout)
}

/*
 * ============================================================================
 * Providers
 * ============================================================================
 */

fn logger_provider(cli: &CliArgs) -> SdkLoggerProvider {
    let mut provider_builder = SdkLoggerProvider::builder().with_resource(resource(cli));

    if let Some(exporter) = &cli.otel_logs_exporter {
        if exporter.contains(&CliArgsOtelExporter::Console) {
            let exporter_builder = opentelemetry_stdout::LogExporter::default();
            provider_builder = provider_builder.with_simple_exporter(exporter_builder);
        }

        if exporter.contains(&CliArgsOtelExporter::Otlp) {
            let compression = logs_compression(cli);
            let protocol = logs_protocol(cli);
            let endpoint = logs_endpoint(cli, protocol);
            let headers = log_headers(cli);
            let timeout = logs_timeout(cli);

            match protocol {
                Protocol::Grpc => {
                    let mut exporter_builder = opentelemetry_otlp::LogExporter::builder()
                        .with_tonic()
                        .with_endpoint(endpoint)
                        .with_metadata(parse_headers_metadata_map(headers))
                        .with_protocol(Protocol::Grpc)
                        .with_timeout(Duration::from_millis(timeout));

                    if let Some(compression) = compression {
                        exporter_builder = exporter_builder.with_compression(compression);
                    }

                    provider_builder =
                        provider_builder.with_batch_exporter(exporter_builder.build().unwrap());
                }
                Protocol::HttpBinary | Protocol::HttpJson => {
                    let exporter_builder = opentelemetry_otlp::LogExporter::builder()
                        .with_http()
                        .with_endpoint(endpoint)
                        .with_headers(parse_headers_hashmap(headers))
                        .with_protocol(protocol)
                        .with_timeout(Duration::from_millis(timeout));

                    provider_builder =
                        provider_builder.with_batch_exporter(exporter_builder.build().unwrap());
                }
            }
        }
    }

    provider_builder.build()
}

fn meter_provider(cli: &CliArgs) -> SdkMeterProvider {
    let mut provider_builder = SdkMeterProvider::builder().with_resource(resource(cli));

    if let Some(exporter) = &cli.otel_metrics_exporter {
        if exporter.contains(&CliArgsOtelExporter::Console) {
            let exporter_builder = opentelemetry_stdout::MetricExporterBuilder::default();
            provider_builder = provider_builder.with_periodic_exporter(exporter_builder.build());
        }

        if exporter.contains(&CliArgsOtelExporter::Otlp) {
            let compression = metrics_compression(cli);
            let protocol = metrics_protocol(cli);
            let endpoint = metrics_endpoint(cli, protocol);
            let headers = metrics_headers(cli);
            let timeout = metrics_timeout(cli);

            match protocol {
                Protocol::Grpc => {
                    let mut exporter_builder = opentelemetry_otlp::MetricExporter::builder()
                        .with_tonic()
                        .with_endpoint(endpoint)
                        .with_metadata(parse_headers_metadata_map(headers))
                        .with_protocol(Protocol::Grpc)
                        .with_timeout(Duration::from_millis(timeout));

                    if let Some(compression) = compression {
                        exporter_builder = exporter_builder.with_compression(compression);
                    }

                    provider_builder =
                        provider_builder.with_periodic_exporter(exporter_builder.build().unwrap());
                }
                Protocol::HttpBinary | Protocol::HttpJson => {
                    let exporter_builder = opentelemetry_otlp::MetricExporter::builder()
                        .with_http()
                        .with_endpoint(endpoint)
                        .with_headers(parse_headers_hashmap(headers))
                        .with_protocol(protocol)
                        .with_timeout(Duration::from_millis(timeout));

                    provider_builder =
                        provider_builder.with_periodic_exporter(exporter_builder.build().unwrap());
                }
            }
        }
    }

    provider_builder.build()
}

fn tracer_provider(cli: &CliArgs) -> SdkTracerProvider {
    let mut provider_builder = SdkTracerProvider::builder().with_resource(resource(cli));

    if let Some(exporter) = &cli.otel_traces_exporter {
        if exporter.contains(&CliArgsOtelExporter::Console) {
            let exporter_builder = opentelemetry_stdout::SpanExporter::default();
            provider_builder = provider_builder.with_simple_exporter(exporter_builder);
        }

        if exporter.contains(&CliArgsOtelExporter::Otlp) {
            let compression = traces_compression(cli);
            let protocol = traces_protocol(cli);
            let endpoint = traces_endpoint(cli, protocol);
            let headers = traces_headers(cli);
            let timeout = traces_timeout(cli);

            match protocol {
                Protocol::Grpc => {
                    let mut exporter_builder = opentelemetry_otlp::SpanExporter::builder()
                        .with_tonic()
                        .with_endpoint(endpoint)
                        .with_metadata(parse_headers_metadata_map(headers))
                        .with_protocol(Protocol::Grpc)
                        .with_timeout(Duration::from_millis(timeout));

                    if let Some(compression) = compression {
                        exporter_builder = exporter_builder.with_compression(compression);
                    }

                    provider_builder =
                        provider_builder.with_batch_exporter(exporter_builder.build().unwrap());
                }
                Protocol::HttpBinary | Protocol::HttpJson => {
                    let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
                        .with_http()
                        .with_endpoint(endpoint)
                        .with_headers(parse_headers_hashmap(headers))
                        .with_protocol(protocol)
                        .with_timeout(Duration::from_millis(timeout));

                    provider_builder =
                        provider_builder.with_batch_exporter(exporter_builder.build().unwrap());
                }
            }
        }
    }

    provider_builder.build()
}

/*
 * ============================================================================
 * Parsers
 * ============================================================================
 */

fn parse_headers_hashmap(headers: Option<&str>) -> HashMap<String, String> {
    headers
        .map(|headers| {
            parse_headers(headers)
                .map(|(key, value)| (key.into(), value.into()))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_headers_metadata_map(headers: Option<&str>) -> MetadataMap {
    headers
        .map(|headers| {
            MetadataMap::from_headers(
                parse_headers(headers)
                    .map(|(key, value)| (key.parse().unwrap(), value.parse().unwrap()))
                    .collect(),
            )
        })
        .unwrap_or_default()
}

fn parse_headers<'a>(
    headers: &'a str,
) -> std::iter::Map<std::str::Split<'a, char>, impl FnMut(&'a str) -> (&'a str, &'a str)> {
    headers.split(',').map(|header| {
        let mut parts = header.splitn(2, '=');
        let key = parts.next().unwrap();
        let value = parts.next().unwrap_or("");
        (key, value)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_headers_hashmap_none() {
        let headers = None;

        let actual = parse_headers_hashmap(headers);

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn parse_headers_hashmap_some() {
        let headers = Some("key1=value1,key2=value2,key3=");

        let actual = parse_headers_hashmap(headers);

        assert_eq!(actual.len(), 3);
        assert_eq!(actual.get("key1").unwrap(), "value1");
        assert_eq!(actual.get("key2").unwrap(), "value2");
        assert_eq!(actual.get("key3").unwrap(), "");
    }

    #[test]
    fn parse_headers_metadata_map_none() {
        let headers = None;

        let actual = parse_headers_metadata_map(headers);

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn parse_headers_metadata_map_some() {
        let headers = Some("key1=value1,key2=value2,key3=");

        let actual = parse_headers_metadata_map(headers);

        assert_eq!(actual.len(), 3);
        assert_eq!(actual.get("key1").unwrap(), "value1");
        assert_eq!(actual.get("key2").unwrap(), "value2");
        assert_eq!(actual.get("key3").unwrap(), "");
    }
}
