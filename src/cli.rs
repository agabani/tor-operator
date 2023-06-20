use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/*
 * ============================================================================
 * Cli
 * ============================================================================
 */
#[allow(clippy::module_name_repetitions)]
#[derive(Parser, Debug)]
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
    /// Run
    Run(ControllerRunArgs),
}

#[derive(Args, Debug)]
pub struct ControllerRunArgs {
    #[arg(long, default_value = "IfNotPresent")]
    pub onion_balance_image_pull_policy: String,

    #[arg(
        long,
        default_value = "ghcr.io/agabani/tor-operator:onion-balance-0.2.2"
    )]
    pub onion_balance_image_uri: String,

    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, default_value_t = 8080)]
    pub port: u16,

    #[arg(long, default_value = "IfNotPresent")]
    pub tor_image_pull_policy: String,

    #[arg(long, default_value = "ghcr.io/agabani/tor-operator:tor-0.4.7.13")]
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
    /// Generate
    Generate(CrdGenerateArgs),
}

#[derive(Args, Debug)]
pub struct CrdGenerateArgs {
    #[arg(long, value_enum, default_value_t = CrdGenerateArgsFormat::Yaml)]
    pub format: CrdGenerateArgsFormat,

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
    /// Generate
    Generate(OnionKeyGenerateArgs),
}

#[derive(Args, Debug)]
pub struct OnionKeyGenerateArgs {}