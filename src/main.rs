use anyhow::{Context, Result};
use clap::Parser;
use home::home_dir;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// List installed JetBrains IDEs
    List {
        /// Show all found IDE paths even if no log file is present
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Debug)]
struct IdeInfo {
    name: String,
    path: PathBuf,
    has_log: bool,
}

fn get_jetbrains_base_path() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        home_dir().map(|h| h.join("Library/Logs/JetBrains"))
    } else if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").ok().map(|p| PathBuf::from(p).join("JetBrains"))
    } else {
        // Linux
        home_dir().map(|h| h.join(".cache/JetBrains"))
    }
}

fn find_ide_installations() -> Result<Vec<IdeInfo>> {
    let base_path = get_jetbrains_base_path()
        .context("Could not determine JetBrains base path")?;

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let mut ides = Vec::new();

    for entry in WalkDir::new(&base_path).max_depth(1).min_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Check if this is a valid IDE directory (contains idea.log or will contain it)
        let log_path = if cfg!(target_os = "macos") {
            path.to_path_buf()
        } else {
            path.join("log")
        };

        let has_log = log_path.join("idea.log").exists();

        ides.push(IdeInfo {
            name,
            path: path.to_path_buf(),
            has_log,
        });
    }

    Ok(ides)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { verbose } => {
            let ides = find_ide_installations()?;

            if ides.is_empty() {
                println!("No JetBrains IDEs found");
                return Ok(());
            }

            println!("Found JetBrains IDEs:");
            for ide in ides {
                if verbose || ide.has_log {
                    println!(
                        "{}: {}{}",
                        ide.name,
                        ide.path.display(),
                        if !ide.has_log { " (no log file present)" } else { "" }
                    );
                }
            }
        }
    }

    Ok(())
}
