mod cli;
mod commands;
mod types;
mod context;
mod errors;

use std::process;
use clap::Parser;
use cli::FrogCli;
use crate::cli::Commands;
use crate::errors::ContainerError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = FrogCli::parse();

    let run_dir = cli.run_dir.unwrap_or_else(|| "/run/frogc".to_string());
    let context = context::FrogContext::new(run_dir);

    let err = match cli.command {
        Commands::Create { spec, container_id } => {
            commands::create::run(context, spec, container_id)
        },
        Commands::Start { container_id } => {
            commands::start::run(container_id)
        }
    };

    if let Err(e) = err {
        eprintln!("Error: {}", e);
        match e {
            ContainerError::AlreadyExists => process::exit(2),
            _ => process::exit(1),
        }
    }

    Ok(())
}
