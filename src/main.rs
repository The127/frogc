mod cli;
mod commands;
mod types;
mod context;

use clap::Parser;
use cli::FrogCli;
use crate::cli::Commands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = FrogCli::parse();

    match cli.command {
        Commands::Create { spec, container_id } => {
            commands::create::run(spec, container_id)?;
        },
        Commands::Start { container_id } => {
            commands::start::run(container_id)?;
        }
    }

    Ok(())
}
