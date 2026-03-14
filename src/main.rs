mod models;
mod output;
mod scanner;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dex", version, about = "Codebase context protocol — generate .dex/ for any project")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a codebase and generate the .dex/ context directory
    Init {
        /// Path to the project root (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let path = path.canonicalize().unwrap_or(path);
            println!(
                "{} {} {}",
                "dex".bold().cyan(),
                "scanning".dimmed(),
                path.display()
            );
            // TODO: scanner + output
            println!("{}", "Done!".bold().green());
        }
    }

    Ok(())
}
