mod cli;
mod server;

use anyhow::Result;
use clap::Parser;
use cli::{AddCommands, Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev { project, port } => {
            let options = cli::dev::DevOptions {
                project_dir: project.unwrap_or_else(|| ".".into()),
                port, // None = auto-select from 3000
            };
            cli::dev::run(options).await?;
        }

        Commands::Init { name } => {
            println!("TODO: Initialize project '{}'", name);
            // TODO: Implement init
        }

        Commands::Add { command } => match command {
            AddCommands::Page { path } => {
                println!("TODO: Add page at '{}'", path);
                // TODO: Implement add page
            }
            AddCommands::Action { path } => {
                println!("TODO: Add action at '{}'", path);
                // TODO: Implement add action
            }
        },

        Commands::Build { project } => {
            let project_dir = project.unwrap_or_else(|| ".".into());
            println!("TODO: Build project at '{}'", project_dir.display());
            // TODO: Implement build
        }
    }

    Ok(())
}
