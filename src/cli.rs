use std::path::PathBuf;

use clap::{Parser, Subcommand};

const DEFAULT_CONFIG_PATH: &str = "josher.toml";

#[derive(Debug, Parser)]
#[command(name = "josher")]
#[command(about = "Synchronize subtree repo with mono repo using Josh")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Path to the config file.
    #[arg(long, default_value = DEFAULT_CONFIG_PATH, global = true)]
    pub config: PathBuf,

    /// Print executed commands.
    #[arg(long, global = true)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a config file for this repository.
    Init,

    /// Pull history from mono repo.
    Pull,

    /// Push history to mono repo.
    Push,
    ClearRefs,
}
