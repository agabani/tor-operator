use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// CRD
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
pub fn parse() -> Cli {
    Cli::parse()
}
