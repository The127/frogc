mod cli;
mod commands;
mod types;

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
        Commands::Start { .. } => {
            println!("Starting container");
        }
    }

    Ok(())
}
