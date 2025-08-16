use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/*
 * ============================================================================
 * Cli
 * ============================================================================
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Parser, Debug)]
#[command(about, version)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommands,

    /// Sets the value of the service.name resource attribute.
    #[arg(long, env, default_value = "tor-operator")]
    pub otel_service_name: String,

    /// Key-value pairs to be used as resource attributes.
    #[arg(long, env)]
    pub otel_resource_attributes: Option<String>,

    /// Specifies which exporters are used for logs.
    #[arg(long, env, value_delimiter = ',')]
    pub otel_logs_exporter: Option<Vec<CliArgsOtelExporter>>,

    /// Specifies which exporters are used for metrics.
    #[arg(long, env, value_delimiter = ',')]
    pub otel_metrics_exporter: Option<Vec<CliArgsOtelExporter>>,

    /// Specifies which exporters are used for traces.
    #[arg(long, env, value_delimiter = ',')]
    pub otel_traces_exporter: Option<Vec<CliArgsOtelExporter>>,

    /// Specifies the OTLP transport compression to be used for all telemetry data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_compression: Option<CliArgsOtelExporterOtlpCompression>,

    /// A base endpoint URL for any signal type, with an optionally-specified port number. Helpful for when you’re sending more than one signal to the same endpoint and want one environment variable to control the endpoint.
    #[arg(long, env)]
    pub otel_exporter_otlp_endpoint: Option<String>,

    /// A list of headers to apply to all outgoing data (traces, metrics, and logs).
    #[arg(long, env)]
    pub otel_exporter_otlp_headers: Option<String>,

    /// Specifies the OTLP transport protocol to be used for all telemetry data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_protocol: Option<CliArgsOtelExporterOtlpProtocol>,

    /// The timeout value for all outgoing data (traces, metrics, and logs) in milliseconds.
    #[arg(long, env, default_value_t = 10000)]
    pub otel_exporter_otlp_timeout: u64,

    /// Specifies the OTLP transport compression to be used for log data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_logs_compression: Option<CliArgsOtelExporterOtlpCompression>,

    /// Endpoint URL for log data only, with an optionally-specified port number. Typically ends with `v1/logs` when using OTLP/HTTP.
    #[arg(long, env)]
    pub otel_exporter_otlp_logs_endpoint: Option<String>,

    /// A list of headers to apply to all outgoing logs.
    #[arg(long, env)]
    pub otel_exporter_otlp_logs_headers: Option<String>,

    /// Specifies the OTLP transport protocol to be used for log data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_logs_protocol: Option<CliArgsOtelExporterOtlpProtocol>,

    /// The timeout value for all outgoing logs in milliseconds.
    #[arg(long, env)]
    pub otel_exporter_otlp_logs_timeout: Option<u64>,

    /// Specifies the OTLP transport compression to be used for metrics data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_metrics_compression: Option<CliArgsOtelExporterOtlpCompression>,

    /// Endpoint URL for metric data only, with an optionally-specified port number. Typically ends with `v1/metrics` when using OTLP/HTTP.
    #[arg(long, env)]
    pub otel_exporter_otlp_metrics_endpoint: Option<String>,

    /// A list of headers to apply to all outgoing metrics.
    #[arg(long, env)]
    pub otel_exporter_otlp_metrics_headers: Option<String>,

    /// Specifies the OTLP transport protocol to be used for metrics data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_metrics_protocol: Option<CliArgsOtelExporterOtlpProtocol>,

    /// The timeout value for all outgoing metrics in milliseconds.
    #[arg(long, env)]
    pub otel_exporter_otlp_metrics_timeout: Option<u64>,

    /// Specifies the OTLP transport compression to be used for trace data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_traces_compression: Option<CliArgsOtelExporterOtlpCompression>,

    /// Endpoint URL for metric data only, with an optionally-specified port number. Typically ends with `v1/traces` when using OTLP/HTTP.
    #[arg(long, env)]
    pub otel_exporter_otlp_traces_endpoint: Option<String>,

    /// A list of headers to apply to all outgoing traces.
    #[arg(long, env)]
    pub otel_exporter_otlp_traces_headers: Option<String>,

    /// Specifies the OTLP transport protocol to be used for trace data.
    #[arg(long, value_enum, env)]
    pub otel_exporter_otlp_traces_protocol: Option<CliArgsOtelExporterOtlpProtocol>,

    /// The timeout value for all outgoing traces in milliseconds.
    #[arg(long, env)]
    pub otel_exporter_otlp_traces_timeout: Option<u64>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum CliArgsOtelExporterOtlpCompression {
    #[value(name = "gzip")]
    Gzip,
    #[value(name = "zstd")]
    Zstd,
}

impl From<CliArgsOtelExporterOtlpCompression> for opentelemetry_otlp::Compression {
    fn from(value: CliArgsOtelExporterOtlpCompression) -> Self {
        match value {
            CliArgsOtelExporterOtlpCompression::Gzip => opentelemetry_otlp::Compression::Gzip,
            CliArgsOtelExporterOtlpCompression::Zstd => opentelemetry_otlp::Compression::Zstd,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum CliArgsOtelExporterOtlpProtocol {
    #[value(name = "grpc")]
    Grpc,
    #[value(name = "http/json")]
    HttpJson,
    #[value(name = "http/protobuf")]
    HttpProtobuf,
}

impl From<CliArgsOtelExporterOtlpProtocol> for opentelemetry_otlp::Protocol {
    fn from(value: CliArgsOtelExporterOtlpProtocol) -> Self {
        match value {
            CliArgsOtelExporterOtlpProtocol::Grpc => opentelemetry_otlp::Protocol::Grpc,
            CliArgsOtelExporterOtlpProtocol::HttpJson => opentelemetry_otlp::Protocol::HttpJson,
            CliArgsOtelExporterOtlpProtocol::HttpProtobuf => {
                opentelemetry_otlp::Protocol::HttpBinary
            }
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliArgsOtelExporter {
    #[value(name = "console")]
    Console,
    #[value(name = "otlp")]
    Otlp,
}

#[must_use]
pub fn parse() -> CliArgs {
    CliArgs::parse()
}

#[allow(clippy::module_name_repetitions)]
#[derive(Subcommand, Debug)]
pub enum CliCommands {
    /// Controller
    Controller(ControllerArgs),

    /// Custom Resource Definition
    Crd(CrdArgs),

    /// Markdown
    Markdown(MarkdownArgs),

    /// Onion Key
    OnionKey(OnionKeyArgs),
}

/*
 * ============================================================================
 * Controller
 * ============================================================================
 */
#[derive(Args, Debug)]
pub struct ControllerArgs {
    #[command(subcommand)]
    pub command: ControllerCommands,
}

#[derive(Subcommand, Debug)]
pub enum ControllerCommands {
    /// Run the Tor Operator
    Run(ControllerRunArgs),
}

#[derive(Args, Debug)]
pub struct ControllerRunArgs {
    /// Onion Balance image pull policy
    #[arg(long, default_value = "IfNotPresent")]
    pub onion_balance_image_pull_policy: String,

    /// Onion Balance image uri
    #[arg(
        long,
        default_value = "ghcr.io/agabani/tor-operator:onion-balance-0.2.4.0"
    )]
    pub onion_balance_image_uri: String,

    /// Host the web server binds to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port the web server binds to
    #[arg(long, default_value_t = 8080)]
    pub port: u16,

    /// Tor image pull policy
    #[arg(long, default_value = "IfNotPresent")]
    pub tor_image_pull_policy: String,

    /// Tor image uri
    #[arg(long, default_value = "ghcr.io/agabani/tor-operator:tor-0.4.8.17.0")]
    pub tor_image_uri: String,
}

/*
 * ============================================================================
 * Custom Resource Document
 * ============================================================================
 */
#[derive(Args, Debug)]
pub struct CrdArgs {
    #[command(subcommand)]
    pub command: CrdCommands,
}

#[derive(Subcommand, Debug)]
pub enum CrdCommands {
    /// Generate the Tor Operator CRDs
    Generate(CrdGenerateArgs),
}

#[derive(Args, Debug)]
pub struct CrdGenerateArgs {
    /// Format of the CRDs
    #[arg(long, value_enum, default_value_t = CrdGenerateArgsFormat::Yaml)]
    pub format: CrdGenerateArgsFormat,

    /// Output the CRDs into a directory
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    pub output: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum CrdGenerateArgsFormat {
    Helm,
    Json,
    Yaml,
}

/*
 * ============================================================================
 * Markdown
 * ============================================================================
 */
#[derive(Args, Debug)]
#[clap(hide = true)]
pub struct MarkdownArgs {
    #[command(subcommand)]
    pub command: MarkdownCommands,
}

#[derive(Subcommand, Debug)]
pub enum MarkdownCommands {
    /// Generate the CLI help docs
    Generate(MarkdownGenerateArgs),
}

#[derive(Args, Debug)]
pub struct MarkdownGenerateArgs {
    /// Output the CLI help docs to a file
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    pub output: Option<PathBuf>,
}

/*
 * ============================================================================
 * Onion Address
 * ============================================================================
 */
#[derive(Args, Debug)]
pub struct OnionKeyArgs {
    #[command(subcommand)]
    pub command: OnionKeyCommands,
}

#[derive(Subcommand, Debug)]
pub enum OnionKeyCommands {
    /// Generate a random Tor Onion Key
    Generate(OnionKeyGenerateArgs),
}

#[derive(Args, Debug)]
pub struct OnionKeyGenerateArgs {
    /// Output the Onion Keys into a directory
    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    pub output: Option<PathBuf>,
}
