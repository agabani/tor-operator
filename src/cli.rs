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
        default_value = "ghcr.io/agabani/tor-operator:onion-balance-0.2.2"
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
    #[arg(long, default_value = "ghcr.io/agabani/tor-operator:tor-0.4.7.14")]
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
