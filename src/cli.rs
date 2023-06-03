use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[allow(clippy::module_name_repetitions)]
#[derive(Parser, Debug)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Custom Resource Definition
    Crd(CrdArgs),
}

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
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[must_use]
pub fn parse() -> CliArgs {
    CliArgs::parse()
}
