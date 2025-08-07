use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChangementError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Parser)]
#[command(name = "changement")]
#[command(about = "Manage versioning and publishing for packages in your project")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize changement in a new project
    Init,
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init => init_command(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}

fn init_command() -> Result<()> {
    let changes_dir = Path::new(".changes");

    // Create .changes directory if it doesn't exist
    if !changes_dir.exists() {
        fs::create_dir(changes_dir)?;
        println!("Created .changes directory");
    } else {
        println!(".changes directory already exists");
    }

    let config_path = changes_dir.join("config.json");

    // Create config.json if it doesn't exist
    if !config_path.exists() {
        let config_content = r#"{
  "version": 1,
  "ignore": []
}"#;
        fs::write(&config_path, config_content)?;
        println!("Created .changes/config.json");
    } else {
        println!(".changes/config.json already exists");
    }

    println!("changement initialized successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(true, true);
    }

    #[test]
    fn test_changement_error_display() {
        let error = ChangementError::Config("test config error".to_string());
        assert_eq!(error.to_string(), "Configuration error: test config error");
        
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = ChangementError::Io(io_error);
        assert_eq!(error.to_string(), "IO error: file not found");
    }
}
