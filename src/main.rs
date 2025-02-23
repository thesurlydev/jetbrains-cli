use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use home::home_dir;
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

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
        /// Output format (text or json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
    /// Show configuration details for a specific IDE
    Config {
        /// Name of the IDE (e.g. "IntelliJIdea2024.3")
        #[arg(long)]
        name: String,
        /// Output format (text or json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
}

#[derive(Debug, Serialize)]
struct JsonOutput<T> {
    tools: T,
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
    vmoptions: Option<Vec<String>>,
    port: Option<u16>,
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

fn read_port_from_file(port_file: &Path) -> Option<u16> {
    std::fs::read_to_string(port_file)
        .ok()
        .and_then(|content| content.trim().parse().ok())
}

fn get_port_file_path(vmoptions: &[String]) -> Option<PathBuf> {
    vmoptions.iter()
        .find(|line| line.starts_with("-Dtoolbox.notification.portFile="))
        .map(|line| {
            let path = line.trim_start_matches("-Dtoolbox.notification.portFile=");
            PathBuf::from(path)
        })
}

fn read_vmoptions(config_dir: &Path, name: &str) -> Option<Vec<String>> {
    let (_, vmoptions_prefix) = get_product_info(name);
    let vmoptions_file = config_dir.join(format!("{}.vmoptions", vmoptions_prefix));
    if !vmoptions_file.exists() {
        return None;
    }

    std::fs::read_to_string(vmoptions_file)
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .map(String::from)
                .collect()
        })
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

fn get_product_info(dir_name: &str) -> (String, String) {
    let (display_name, vmoptions_prefix) = match dir_name {
        name if name.starts_with("IntelliJIdea") => ("IntelliJ IDEA", "idea"),
        name if name.starts_with("WebStorm") => ("WebStorm", "webstorm"),
        name if name.starts_with("RustRover") => ("RustRover", "rustrover"),
        name if name.starts_with("CLion") => ("CLion", "clion"),
        name if name.starts_with("PyCharm") => ("PyCharm", "pycharm"),
        name if name.starts_with("GoLand") => ("GoLand", "goland"),
        name if name.starts_with("PhpStorm") => ("PhpStorm", "phpstorm"),
        name if name.starts_with("Rider") => ("Rider", "rider"),
        name if name.starts_with("DataGrip") => ("DataGrip", "datagrip"),
        _ => (dir_name, dir_name),
    };
    (display_name.to_string(), vmoptions_prefix.to_lowercase())
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
            
        let (app_name, _) = get_product_info(&dir_name);

        let logs_dir = if cfg!(target_os = "macos") {
            path.to_path_buf()
        } else {
            path.join("log")
        };

        let config_dir = find_ide_config_dir(&dir_name);
        let vmoptions = read_vmoptions(&config_dir, &dir_name);
        
        let port = vmoptions.as_ref()
            .and_then(|opts| get_port_file_path(opts))
            .and_then(|port_file| read_port_from_file(&port_file));
        
        ides.push(IdeInfo {
            name: dir_name.clone(),
            logs_dir,
            install_dir: find_ide_install_dir(&app_name),
            config_dir,
            vmoptions,
            port,
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
                tools: Vec::<IdeInfo>::new()
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
                tools: filtered_ides
            })?)
        }
    }

    Ok(())
}

fn output_ide_config(format: OutputFormat, ide: IdeInfo) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!("Configuration for {}:", ide.name);
            println!("  Install directory: {}", ide.install_dir.display());
            println!("  Config directory: {}", ide.config_dir.display());
            println!("  Logs directory: {}", ide.logs_dir.display());
            if let Some(vmoptions) = ide.vmoptions {
                println!("  VM Options:");
                for opt in vmoptions {
                    println!("    {}", opt);
                }
            } else {
                println!("  VM Options: Not found");
            }
            if let Some(port) = ide.port {
                println!("  Port: {}", port);
            } else {
                println!("  Port: Not found");
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&JsonOutput {
                tools: ide
            })?)
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { verbose, output } => {
            let ides = find_ide_installations()?;
            output_ides(output, ides, verbose)?
        }
        Commands::Config { name, output } => {
            let ides = find_ide_installations()?;
            let ide = ides.into_iter()
                .find(|ide| ide.name == name)
                .ok_or_else(|| anyhow::anyhow!("IDE '{}' not found", name))?;
            output_ide_config(output, ide)?
        }
    }

    Ok(())
}
