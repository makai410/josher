use anyhow::Context;
use clap::Parser;
use josher::cli::{Cli, Command};
use josher::config::Config;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let config = Config::template();
            config
                .write(&cli.config)
                .context("cannot initialize config file")?;
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
        Command::ClearRefs => todo!(),
    }
}
