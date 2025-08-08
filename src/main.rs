use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "changement")]
#[command(about = "Manage versioning and publishing for packages in your project")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize changement in a new project
    Init {
        /// The path to the project directory where changement should be initialized
        #[arg(default_value = ".")]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", if cli.verbose { "debug" } else { "info" });

    env_logger::Builder::from_env(env)
        .format(|buf, record| match record.level() {
            log::Level::Info => writeln!(buf, "{}", record.args()),
            _ => writeln!(buf, "[{}] {}", record.level(), record.args()),
        })
        .format_timestamp(None)
        // .format(|buf, record| writeln!(buf, "hello"))
        .init();

    if let Err(e) = process(cwd, &cli.command) {
        error!("Error: {e:?}");
        std::process::exit(1);
    }
}

fn process(cwd: PathBuf, command: &Command) -> Result<()> {
    match command {
        Command::Init { path } => init_command(cwd, path),
    }
}

fn init_command(cwd: PathBuf, path: &str) -> Result<()> {
    let path = cwd.join(path);
    let changelog_dir = path.join(".changelog");

    if !changelog_dir.exists() {
        fs::create_dir(&changelog_dir)?;
        info!("Created .changelog directory");
    } else {
        debug!(
            ".changelog directory already exists at {}",
            changelog_dir.canonicalize()?.display()
        );
    }

    let config_path = changelog_dir.join("config.yml");
    if !config_path.exists() {
        let contents = serde_yml::to_string(&Config::default())?;
        fs::write(&config_path, contents)?;
        info!("Created .changelog/config.yml");
    } else {
        debug!(
            ".changelog/config.yml already exists at {}",
            config_path.canonicalize()?.display()
        );
    }

    info!("Changement initialized successfully!");

    Ok(())
}

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq)]
struct Config {
    ignore: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum ChangementError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Configuration error: {0}")]
        Config(String),
    }

    impl Config {
        fn parse(contents: String) -> Result<Self, ChangementError> {
            serde_yml::from_str(&contents).map_err(|e| ChangementError::Config(e.to_string()))
        }
    }

    #[test]
    fn test_init_command_creates_changelog_if_does_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        let cmd = Command::Init {
            path: String::from("."),
        };

        process(cwd, &cmd).unwrap();

        assert!(temp_dir.path().join(".changelog").exists());
        assert!(temp_dir.path().join(".changelog/config.yml").exists());

        let config = Config::parse(
            fs::read_to_string(temp_dir.path().join(".changelog/config.yml")).unwrap(),
        )
        .unwrap();

        assert_eq!(config, Config::default());
    }
}
