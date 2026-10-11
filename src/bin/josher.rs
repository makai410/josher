use anyhow::Context;
use clap::Parser;
use josher::cli::{Cli, Command};
use josher::config::Config;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let config = Config {
                mono_filter: "<mono-filter>".to_string(),
                subtree_filter: "<subtree-filter>".to_string(),
            };
            config
                .write(&cli.config)
                .context("cannot initialize confige file")?;
            println!("Created config file at {}", cli.config.display());
            Ok(())
        }
        Command::Pull => {
            let _config = Config::load(&cli.config)?;
            todo!()
        }
        Command::Push => {
            let _config = Config::load(&cli.config)?;
            todo!()
        }
    }
}
