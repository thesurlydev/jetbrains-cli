use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use home::home_dir;
use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output format (text or json)
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,

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

#[derive(Debug, Serialize)]
struct JsonOutput<T> {
    data: T,
}

#[derive(Debug, Serialize)]
struct IdeInfo {
    name: String,
    #[serde(serialize_with = "serialize_path")]
    logs_dir: PathBuf,
    #[serde(serialize_with = "serialize_path")]
    install_dir: PathBuf,
    #[serde(serialize_with = "serialize_path")]
    config_dir: PathBuf,
}

// Custom serializer for PathBuf to ensure it's always a string in JSON
fn serialize_path<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&path.display().to_string())
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

fn find_ide_config_dir(name: &str) -> PathBuf {
    if cfg!(target_os = "macos") {
        home_dir().unwrap_or_default()
            .join("Library/Application Support/JetBrains")
            .join(name)
    } else if cfg!(target_os = "windows") {
        PathBuf::from("%APPDATA%")
            .join("JetBrains")
            .join(name)
    } else {
        // Linux
        home_dir().unwrap_or_default()
            .join(".config/JetBrains")
            .join(name)
    }
}

fn find_ide_install_dir(name: &str) -> PathBuf {
    if cfg!(target_os = "macos") {
        // Check common installation paths on macOS
        let app_name = if name.ends_with(".app") {
            name.to_string()
        } else {
            format!("{}.app", name)
        };
        
        let paths = [
            PathBuf::from("/Applications").join(&app_name),
            home_dir().unwrap_or_default().join("Applications").join(&app_name),
        ];

        for path in paths {
            if path.exists() {
                return path;
            }
        }
        
        // Return the standard /Applications path even if not found
        PathBuf::from("/Applications").join(app_name)
    } else if cfg!(target_os = "windows") {
        // Default to Program Files on Windows
        PathBuf::from(r"C:\Program Files\JetBrains").join(name)
    } else {
        // Default to opt on Linux
        PathBuf::from("/opt/jetbrains").join(name)
    }
}

fn map_log_dir_to_app_name(dir_name: &str) -> String {
    match dir_name {
        "IntelliJIdea2024.3" => "IntelliJ IDEA",
        "WebStorm2024.3" => "WebStorm",
        "RustRover2024.3" => "RustRover",
        "CLion2024.3" => "CLion",
        "PyCharm2024.3" => "PyCharm",
        "GoLand2024.3" => "GoLand",
        "PhpStorm2024.3" => "PhpStorm",
        "Rider2024.3" => "Rider",
        "DataGrip2024.3" => "DataGrip",
        _ => dir_name,
    }.to_string()
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
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
            
        let app_name = map_log_dir_to_app_name(&dir_name);

        let logs_dir = if cfg!(target_os = "macos") {
            path.to_path_buf()
        } else {
            path.join("log")
        };

        ides.push(IdeInfo {
            name: dir_name.clone(),
            logs_dir,
            install_dir: find_ide_install_dir(&app_name),
            config_dir: find_ide_config_dir(&dir_name),
        });
    }

    Ok(ides)
}

fn output_ides(format: OutputFormat, ides: Vec<IdeInfo>, verbose: bool) -> Result<()> {
    let filtered_ides = if verbose {
        ides
    } else {
        ides.into_iter()
            .filter(|ide| ide.logs_dir.join("idea.log").exists())
            .collect()
    };

    if filtered_ides.is_empty() {
        match format {
            OutputFormat::Text => println!("No JetBrains IDEs found"),
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&JsonOutput {
                data: Vec::<IdeInfo>::new()
            })?),
        }
        return Ok(());
    }

    match format {
        OutputFormat::Text => {
            println!("Found JetBrains IDEs:");
            for ide in filtered_ides {
                println!(
                    "{}: {}",
                    ide.name,
                    ide.install_dir.display()
                );
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&JsonOutput {
                data: filtered_ides
            })?)
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { verbose } => {
            let ides = find_ide_installations()?;
            output_ides(cli.output, ides, verbose)?
        }
    }

    Ok(())
}
