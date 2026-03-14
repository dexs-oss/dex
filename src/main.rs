mod models;
mod output;
mod scanner;
mod show;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "dex",
    version,
    about = "Codebase context protocol — generate .dex/ for any project"
)]
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
    /// Display project context from .dex/ directory
    Show {
        /// Path to the project root (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Show only a specific section (project, structure, entry-points, api)
        #[arg(long)]
        section: Option<String>,
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

            let result = scanner::scan(&path)?;

            println!(
                "  {} {}",
                "languages:".dimmed(),
                result.context.project.languages.join(", ")
            );
            println!(
                "  {} {}",
                "type:".dimmed(),
                result.context.project.project_type
            );
            println!(
                "  {} {}",
                "entry points:".dimmed(),
                result.paths.entry_points.len()
            );

            output::write_dex_dir(&path, &result)?;

            println!(
                "\n{} Generated {} in {}/",
                "done".bold().green(),
                ".dex/".bold(),
                path.display()
            );
        }
        Commands::Show { path, section } => {
            let path = path.canonicalize().unwrap_or(path);
            show::show(&path, section.as_deref())?;
        }
    }

    Ok(())
}
