use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, info};
use std::fs;
use std::path::Path;
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
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize changement in a new project
    Init,
    /// Create a new change for a package in your project
    New {
        /// The name of the package to create the change for
        #[arg(short, long)]
        package: String,
        /// The message for the change
        #[arg(short, long)]
        message: String,
        /// The type of version bump (major, minor, patch)
        #[arg(short, long)]
        bump: BumpType,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logger based on verbose flag
    init_logger(cli.verbose);

    let result = match &cli.command {
        Commands::Init => init_command(),
        Commands::New {
            package,
            message,
            bump,
        } => new_command(package, message, bump),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn init_logger(verbose: bool) {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        if verbose {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Debug)
                .init();
        } else {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Info)
                .init();
        }
    });
}

fn init_command() -> Result<()> {
    debug!("Starting init command");
    let changes_dir = Path::new(".changes");

    // Create .changes directory if it doesn't exist
    if !changes_dir.exists() {
        fs::create_dir(changes_dir).context("Failed to create .changes directory")?;
        info!("Created .changes directory");
    } else {
        info!(".changes directory already exists");
    }

    let config_path = changes_dir.join("config.json");

    // Create config.json if it doesn't exist
    if !config_path.exists() {
        let config_content = r#"{
  "version": 1,
  "ignore": []
}"#;
        fs::write(&config_path, config_content).context("Failed to write config.json")?;
        info!("Created .changes/config.json");
    } else {
        info!(".changes/config.json already exists");
    }

    println!("changement initialized successfully!");
    Ok(())
}

fn new_command(package: &str, message: &str, bump: &BumpType) -> Result<()> {
    debug!("Starting new command for package: {package}");

    let changes_dir = Path::new(".changes");

    // Ensure .changes directory exists
    if !changes_dir.exists() {
        anyhow::bail!(".changes directory does not exist. Run 'changement init' first.");
    }

    // Generate a unique filename for this change
    let change_id = uuid::Uuid::new_v4();
    let filename = format!("{package}-{change_id}.md");
    let file_path = changes_dir.join(filename);

    debug!("Creating change file: {}", file_path.display());

    // Convert bump type to lowercase string
    let bump_str = match bump {
        BumpType::Major => "major",
        BumpType::Minor => "minor",
        BumpType::Patch => "patch",
    };

    // Create the markdown content with YAML frontmatter
    let content = format!("---\n\"{package}\": {bump_str}\n---\n\n{message}\n");

    // Write the change file
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write change file: {}", file_path.display()))?;

    info!("Created change file: {}", file_path.display());
    println!("Created change for package '{package}' with {bump_str} bump");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Use a global mutex to serialize tests that change the current directory
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test() {
        init_logger(false);
    }

    #[test]
    fn test_init_command_creates_directory_and_config() {
        setup_test();
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = init_command();
        assert!(result.is_ok());

        // Check that .changes directory was created
        assert!(Path::new(".changes").exists());

        // Check that config.json was created with correct content
        let config_path = Path::new(".changes/config.json");
        assert!(config_path.exists());

        let config_content = fs::read_to_string(config_path).unwrap();
        assert!(config_content.contains("\"version\": 1"));
        assert!(config_content.contains("\"ignore\": []"));

        std::env::set_current_dir(old_dir).unwrap();
    }

    #[test]
    fn test_new_command_creates_change_file() {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_test();
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // First initialize the project
        init_command().unwrap();

        // Create a new change
        let result = new_command("test-package", "Test message", &BumpType::Minor);
        assert!(result.is_ok());

        // Check that a change file was created
        let changes_dir = Path::new(".changes");
        let entries: Vec<_> = fs::read_dir(changes_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 1);

        // Check the content of the created file
        let content = fs::read_to_string(&entries[0]).unwrap();
        assert!(content.contains("\"test-package\": minor"));
        assert!(content.contains("Test message"));
        assert!(content.starts_with("---\n"));

        std::env::set_current_dir(old_dir).unwrap();
    }

    #[test]
    fn test_new_command_without_init_fails() {
        let _guard = TEST_MUTEX.lock().unwrap();
        setup_test();
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Try to create a new change without initializing first
        let result = new_command("test-package", "Test message", &BumpType::Patch);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains(".changes directory does not exist")
        );

        std::env::set_current_dir(old_dir).unwrap();
    }

    #[test]
    fn test_bump_types() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        setup_test();
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        init_command().unwrap();

        // Test each bump type
        new_command("pkg1", "Major change", &BumpType::Major).unwrap();
        new_command("pkg2", "Minor change", &BumpType::Minor).unwrap();
        new_command("pkg3", "Patch change", &BumpType::Patch).unwrap();

        let changes_dir = Path::new(".changes");
        let entries: Vec<_> = fs::read_dir(changes_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "md" {
                    Some(fs::read_to_string(path).ok()?)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 3);
        assert!(
            entries
                .iter()
                .any(|content| content.contains("\"pkg1\": major"))
        );
        assert!(
            entries
                .iter()
                .any(|content| content.contains("\"pkg2\": minor"))
        );
        assert!(
            entries
                .iter()
                .any(|content| content.contains("\"pkg3\": patch"))
        );

        std::env::set_current_dir(old_dir).unwrap();
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
