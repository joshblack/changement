use crate::graph::{Node, NodeIndex};
use crate::project::{Project, Workspace};
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

mod graph;
mod project;

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

    /// Create a new changelog entry
    New {
        /// The name of the package
        #[arg(short, long)]
        package: String,

        /// A message describing the changes that will be included in the changelog
        #[arg(short, long)]
        message: String,

        /// The version bump type to apply to the package
        #[arg(short, long, default_value = "minor")]
        bump: VersionBump,
    },

    /// Update the versions of packages based on changelog entries in the .changelog directory
    Version {
        /// A filter to apply to the changelog entries
        #[arg(short, long)]
        filter: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, clap::ValueEnum, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum VersionBump {
    #[value(name = "major")]
    Major,

    #[value(name = "minor")]
    Minor,

    #[value(name = "patch")]
    Patch,
}

impl PartialOrd for VersionBump {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (VersionBump::Major, VersionBump::Major) => Some(std::cmp::Ordering::Equal),
            (VersionBump::Major, _) => Some(std::cmp::Ordering::Greater),
            (_, VersionBump::Major) => Some(std::cmp::Ordering::Less),
            (VersionBump::Minor, VersionBump::Minor) => Some(std::cmp::Ordering::Equal),
            (VersionBump::Minor, _) => Some(std::cmp::Ordering::Greater),
            (_, VersionBump::Minor) => Some(std::cmp::Ordering::Less),
            (VersionBump::Patch, VersionBump::Patch) => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl Ord for VersionBump {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Display for VersionBump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionBump::Major => write!(f, "major"),
            VersionBump::Minor => write!(f, "minor"),
            VersionBump::Patch => write!(f, "patch"),
        }
    }
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
        Command::New {
            package,
            message,
            bump,
        } => new_command(cwd, package, message, bump),
        Command::Version { filter } => version_command(cwd, filter),
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

fn new_command(cwd: PathBuf, package: &str, message: &str, bump: &VersionBump) -> Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut generator = names::Generator::default();
    let filepath = generator
        .find_map(|name| {
            let filepath = cwd
                .join(".changelog")
                .join(format!("{}-{}", &timestamp, name))
                .with_extension("md");
            if filepath.exists() {
                None
            } else {
                Some(filepath)
            }
        })
        .ok_or(anyhow!("Unable to generate name for new changelog entry"))?;
    let changelog_entry = ChangelogEntry {
        frontmatter: HashMap::from([(package.into(), bump.clone())]),
        body: message.to_string(),
    };
    let contents = changelog_entry.to_string()?;

    fs::write(&filepath, contents)?;

    info!(
        "Created new changelog entry: {}",
        filepath.file_name().unwrap().display()
    );

    Ok(())
}

fn version_command(cwd: PathBuf, filter: &Vec<String>) -> Result<()> {
    let project = Project::new(&cwd);
    let changelog_dir = cwd.join(".changelog");
    if !changelog_dir.exists() {
        return Err(anyhow!(
            "Changelog directory does not exist at {}",
            changelog_dir.display()
        ));
    }

    let changelog_entries = get_changelog_entries(&changelog_dir)?;
    let changes = changelog_entries.iter().flat_map(|entry| {
        entry
            .frontmatter
            .iter()
            .filter_map(|(package, bump)| {
                if filter.is_empty() || filter.contains(package) {
                    Some((package, bump))
                } else {
                    None
                }
            })
            .filter_map(|(package, bump)| {
                if let Some((index, _)) = project.get_workspace(package) {
                    Some((index, bump))
                } else {
                    None
                }
            })
    });
    let bumps: HashMap<NodeIndex, &VersionBump> = changes.into_iter().collect();

    for (index, bump) in bumps {
        bump_workspace(&project, index, bump);
    }

    fn bump_workspace(project: &Project, workspace_index: NodeIndex, bump: &VersionBump) {
        let workspace = project
            .workspace(workspace_index)
            .expect("Workspace not found");

        for dependent_index in project.dependents(workspace_index) {
            let dependent = project
                .workspace(dependent_index)
                .expect("Dependent workspace not found");
            let version = dependent.data.dependency_version(&workspace.data.name);
            // if !from_parent_update {
            // bump_workspace(&project, dependent, bump, true);
            // } else {
            // //
            // }
        }
    }

    Ok(())
}

// Filtering
// Coming from pnpm: https://pnpm.io/filtering
// patterns:
//   - wildcard
//   - path
//   - name
//   - dependencies (foo...) matches foo and all of its dependencies
//   - only dependencies (foo^...)
//   - dependents (...foo)
//   - only dependents (...^foo)
//   - glob
//   - [<since>] which could be a branch
//   - exclusion (with a ! in front of a pattern)

struct ChangelogEntry {
    frontmatter: HashMap<String, VersionBump>,
    body: String,
}

fn get_changelog_entries(directory: impl AsRef<Path>) -> Result<Vec<ChangelogEntry>> {
    let changelog_dir = directory.as_ref().join(".changelog");
    if !changelog_dir.exists() {
        return Err(anyhow!(
            "Changelog directory does not exist at {}",
            changelog_dir.display()
        ));
    }
    let mut entries = fs::read_dir(&changelog_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            if entry.path().extension() == Some(std::ffi::OsStr::new("md")) {
                let metadata = std::fs::metadata(entry.path()).expect("Failed to read metadata");
                let contents = fs::read_to_string(entry.path()).expect("Failed to read file");
                let changelog_entry =
                    ChangelogEntry::from_string(contents).expect("Failed to parse changelog entry");
                Some((
                    metadata.created().expect("Failed to get created time"),
                    changelog_entry,
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Sort entries by creation time
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let entries = entries.into_iter().map(|e| e.1).collect::<Vec<_>>();

    Ok(entries)
}

impl ChangelogEntry {
    fn to_string(&self) -> Result<String, serde_yml::Error> {
        let frontmatter_yaml = serde_yml::to_string(&self.frontmatter)?;
        Ok(format!("---\n{}---\n\n{}", frontmatter_yaml, self.body))
    }

    #[allow(dead_code)]
    fn from_string(contents: String) -> Result<Self, serde_yml::Error> {
        let parts: Vec<&str> = contents.trim().split("---").collect();
        if parts.len() < 3 {
            return Err(serde::de::Error::custom("Invalid changelog entry format"));
        }

        let frontmatter: HashMap<String, VersionBump> = serde_yml::from_str(parts[1].trim())?;

        Ok(ChangelogEntry {
            frontmatter,
            body: parts[2].trim().to_string(),
        })
    }
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
        let cli = Cli::try_parse_from(["", "init"]).unwrap();

        process(cwd, &cli.command).unwrap();

        assert!(temp_dir.path().join(".changelog").exists());
        assert!(temp_dir.path().join(".changelog/config.yml").exists());

        let config = Config::parse(
            fs::read_to_string(temp_dir.path().join(".changelog/config.yml")).unwrap(),
        )
        .unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_new_command() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        let cli = Cli::try_parse_from(["", "init"]).unwrap();

        process(cwd.clone(), &cli.command).unwrap();

        let cli = Cli::try_parse_from([
            "",
            "new",
            "--package",
            "test-package",
            "--message",
            "This is a test change",
            "--bump",
            "minor",
        ])?;

        process(cwd.clone(), &cli.command)?;

        let changelog_dir = cwd.join(".changelog");
        let file = fs::read_dir(&changelog_dir)?
            .find_map(|r| match r {
                Ok(entry) => {
                    if entry.path().extension() == Some(std::ffi::OsStr::new("md")) {
                        Some(entry.path())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .unwrap();

        assert!(file.exists());

        let contents = fs::read_to_string(file)?;
        let changelog_entry = ChangelogEntry::from_string(contents)?;

        assert_eq!(
            changelog_entry.frontmatter.get("test-package").unwrap(),
            &VersionBump::Minor
        );
        assert_eq!(changelog_entry.body, "This is a test change");

        Ok(())
    }
}
