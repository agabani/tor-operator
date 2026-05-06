#![warn(clippy::pedantic)]

pub mod cli;
mod collections;
pub mod http_server;
mod kubernetes;
pub mod metrics;
pub mod onion_balance;
pub mod onion_key;
pub mod onion_service;
pub mod otel;
pub mod tor;
pub mod tor_ingress;
pub mod tor_proxy;

/*
 * ============================================================================
 * Error
 * ============================================================================
 */
#[derive(Debug)]
pub enum Error {
    Kube(kube::Error),
    MissingConfiguration(&'static str),
    MissingObjectKey(&'static str),
    OtlpExporter(opentelemetry_otlp::ExporterBuildError),
    SyncInvariantViolated(usize),
}

impl std::error::Error for Error {}

impl From<kube::Error> for Error {
    fn from(e: kube::Error) -> Self {
        Self::Kube(e)
    }
}

impl From<opentelemetry_otlp::ExporterBuildError> for Error {
    fn from(e: opentelemetry_otlp::ExporterBuildError) -> Self {
        Self::OtlpExporter(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kube(e) => write!(f, "kubernetes error: {e}"),
            Self::MissingConfiguration(msg) => write!(f, "missing configuration: {msg}"),
            Self::MissingObjectKey(key) => write!(f, "missing object key: {key}"),
            Self::OtlpExporter(e) => write!(f, "OTLP exporter error: {e}"),
            Self::SyncInvariantViolated(count) => {
                write!(
                    f,
                    "sync invariant violated: {count} resources were not patched"
                )
            }
        }
    }
}

/*
 * ============================================================================
 * Result
 * ============================================================================
 */
pub type Result<T, E = Error> = std::result::Result<T, E>;
