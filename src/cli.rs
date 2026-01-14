use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "frogc")]
#[command(about = "A mutable, CLI-only container runtime", long_about = None)]
pub struct FrogCli {
    #[arg(long)]
    pub run_dir: Option<String>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Create a new container
    Create {
        #[arg(long)]
        spec: String,

        container_id: String,
    },
    Start {
        container_id: String,
    }
}