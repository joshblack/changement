use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use names::{Generator, Name};
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
    /// Create a new change for a package in your project
    New {
        /// The name of the package(s) to create the change for
        #[arg(short, long)]
        package: Vec<String>,
        /// The message for the change
        #[arg(short, long)]
        message: String,
        /// The type of version bump (major, minor, patch)
        #[arg(short, long)]
        bump: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", if cli.verbose { "debug" } else { "info" });

    env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .format(|buf, record| match record.level() {
            log::Level::Info => writeln!(buf, "{}", record.args()),
            _ => writeln!(buf, "[{}] {}", record.level(), record.args()),
        })
        .init();

    if let Err(e) = process(cwd, &cli.command) {
        error!("Error: {e:?}");
        std::process::exit(1);
    }
}

fn process(cwd: PathBuf, command: &Command) -> Result<()> {
    match command {
        Command::Init { path } => init_command(cwd, path),
        Command::New { package, message, bump } => new_command(cwd, package, message, bump),
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

fn new_command(cwd: PathBuf, packages: &[String], message: &str, bump: &str) -> Result<()> {
    // Validate bump type
    if !matches!(bump, "major" | "minor" | "patch") {
        return Err(anyhow::anyhow!(
            "Invalid bump type '{}'. Must be one of: major, minor, patch",
            bump
        ));
    }

    let changelog_dir = cwd.join(".changelog");
    
    // Check if .changelog directory exists
    if !changelog_dir.exists() {
        return Err(anyhow::anyhow!(
            ".changelog directory does not exist. Run 'changement init' first."
        ));
    }

    // Process each package
    for package in packages {
        debug!("Creating new change for package '{}' with bump '{}'", package, bump);

        // Generate unique filename using names crate
        let mut generator = Generator::with_naming(Name::Numbered);
        let random_name = generator.next().unwrap();
        let filename = format!("{}-{}.md", random_name, package.replace(" ", "-"));
        let change_file_path = changelog_dir.join(filename);

        debug!("Creating change file at {}", change_file_path.display());

        // Create YAML frontmatter
        let frontmatter = format!("---\n'{}': {}\n---\n\n{}", package, bump, message);

        fs::write(&change_file_path, frontmatter)?;
        
        info!("Created change file: {}", change_file_path.display());
    }

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

    #[test]
    fn test_new_command_creates_change_file() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // First initialize the project
        let init_cmd = Command::Init {
            path: String::from("."),
        };
        process(cwd.clone(), &init_cmd).unwrap();

        // Then create a new change
        let new_cmd = Command::New {
            package: vec![String::from("test-package")],
            message: String::from("Test change message"),
            bump: String::from("minor"),
        };
        process(cwd.clone(), &new_cmd).unwrap();

        // Check that a change file was created
        let changelog_dir = temp_dir.path().join(".changelog");
        let entries: Vec<_> = fs::read_dir(&changelog_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension()? == "md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 1);
        
        let change_file_path = &entries[0];
        let contents = fs::read_to_string(change_file_path).unwrap();
        
        // Verify the format
        assert!(contents.starts_with("---\n'test-package': minor\n---\n\nTest change message"));
    }

    #[test]
    fn test_new_command_with_major_bump() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Initialize first
        let init_cmd = Command::Init {
            path: String::from("."),
        };
        process(cwd.clone(), &init_cmd).unwrap();

        // Create change with major bump
        let new_cmd = Command::New {
            package: vec![String::from("major-package")],
            message: String::from("Breaking change"),
            bump: String::from("major"),
        };
        process(cwd.clone(), &new_cmd).unwrap();

        // Verify the file was created with correct bump type
        let changelog_dir = temp_dir.path().join(".changelog");
        let entries: Vec<_> = fs::read_dir(&changelog_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension()? == "md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 1);
        
        let change_file_path = &entries[0];
        let contents = fs::read_to_string(change_file_path).unwrap();
        
        assert!(contents.contains("'major-package': major"));
        assert!(contents.contains("Breaking change"));
    }

    #[test]
    fn test_new_command_with_patch_bump() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Initialize first
        let init_cmd = Command::Init {
            path: String::from("."),
        };
        process(cwd.clone(), &init_cmd).unwrap();

        // Create change with patch bump
        let new_cmd = Command::New {
            package: vec![String::from("patch-package")],
            message: String::from("Bug fix"),
            bump: String::from("patch"),
        };
        process(cwd.clone(), &new_cmd).unwrap();

        // Verify the file was created with correct bump type
        let changelog_dir = temp_dir.path().join(".changelog");
        let entries: Vec<_> = fs::read_dir(&changelog_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension()? == "md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 1);
        
        let change_file_path = &entries[0];
        let contents = fs::read_to_string(change_file_path).unwrap();
        
        assert!(contents.contains("'patch-package': patch"));
        assert!(contents.contains("Bug fix"));
    }

    #[test]
    fn test_new_command_fails_with_invalid_bump() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Initialize first
        let init_cmd = Command::Init {
            path: String::from("."),
        };
        process(cwd.clone(), &init_cmd).unwrap();

        // Try to create change with invalid bump type
        let new_cmd = Command::New {
            package: vec![String::from("test-package")],
            message: String::from("Test message"),
            bump: String::from("invalid"),
        };
        
        let result = process(cwd, &new_cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid bump type"));
    }

    #[test]
    fn test_new_command_fails_without_changelog_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();

        // Try to create change without initializing first
        let new_cmd = Command::New {
            package: vec![String::from("test-package")],
            message: String::from("Test message"),
            bump: String::from("minor"),
        };
        
        let result = process(cwd, &new_cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".changelog directory does not exist"));
    }

    #[test]
    fn test_new_command_creates_unique_filenames() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Initialize first
        let init_cmd = Command::Init {
            path: String::from("."),
        };
        process(cwd.clone(), &init_cmd).unwrap();

        // Create two changes for the same package
        let new_cmd1 = Command::New {
            package: vec![String::from("same-package")],
            message: String::from("First change"),
            bump: String::from("minor"),
        };
        process(cwd.clone(), &new_cmd1).unwrap();

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));

        let new_cmd2 = Command::New {
            package: vec![String::from("same-package")],
            message: String::from("Second change"),
            bump: String::from("patch"),
        };
        process(cwd.clone(), &new_cmd2).unwrap();

        // Check that two different files were created
        let changelog_dir = temp_dir.path().join(".changelog");
        let entries: Vec<_> = fs::read_dir(&changelog_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension()? == "md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 2);
        
        // Verify both files have different content
        let mut contents = Vec::new();
        for path in entries {
            contents.push(fs::read_to_string(path).unwrap());
        }
        
        assert!(contents[0] != contents[1]);
        assert!(contents.iter().any(|c| c.contains("First change")));
        assert!(contents.iter().any(|c| c.contains("Second change")));
    }

    #[test]
    fn test_new_command_with_multiple_packages() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Initialize first
        let init_cmd = Command::Init {
            path: String::from("."),
        };
        process(cwd.clone(), &init_cmd).unwrap();

        // Create change for multiple packages
        let new_cmd = Command::New {
            package: vec![
                String::from("package-one"),
                String::from("package-two"),
                String::from("package-three"),
            ],
            message: String::from("Multi-package change"),
            bump: String::from("minor"),
        };
        process(cwd.clone(), &new_cmd).unwrap();

        // Check that three change files were created
        let changelog_dir = temp_dir.path().join(".changelog");
        let entries: Vec<_> = fs::read_dir(&changelog_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension()? == "md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(entries.len(), 3);
        
        // Verify each file has the correct package and content
        let mut contents = Vec::new();
        for path in entries {
            contents.push(fs::read_to_string(path).unwrap());
        }
        
        // Check that each package has its own file with correct content
        assert!(contents.iter().any(|c| c.contains("'package-one': minor")));
        assert!(contents.iter().any(|c| c.contains("'package-two': minor")));
        assert!(contents.iter().any(|c| c.contains("'package-three': minor")));
        
        // All files should contain the same message
        for content in contents {
            assert!(content.contains("Multi-package change"));
        }
    }
}
