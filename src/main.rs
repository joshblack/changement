use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use semver::Version;
use regex::Regex;

#[derive(Serialize, Deserialize, Default, Debug, Eq, PartialEq)]
struct Config {
    ignore: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageJson {
    name: String,
    version: String,
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: HashMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: HashMap<String, String>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

impl BumpType {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "major" => Ok(BumpType::Major),
            "minor" => Ok(BumpType::Minor),
            "patch" => Ok(BumpType::Patch),
            _ => Err(anyhow::anyhow!("Invalid bump type: {}", s)),
        }
    }

    fn is_greater_than(&self, other: &BumpType) -> bool {
        match (self, other) {
            (BumpType::Major, BumpType::Minor | BumpType::Patch) => true,
            (BumpType::Minor, BumpType::Patch) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct ChangeEntry {
    packages: HashMap<String, BumpType>,
    file_path: PathBuf,
}

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
    /// Apply all changes to your project and update the versions of packages
    Version {
        /// Filter packages to create changes for (e.g. package-a,package-b)
        #[arg(long)]
        filter: Option<String>,
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
        Command::Version { filter } => version_command(cwd, filter.as_deref()),
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

fn version_command(cwd: PathBuf, filter: Option<&str>) -> Result<()> {
    let changelog_dir = cwd.join(".changelog");
    
    if !changelog_dir.exists() {
        return Err(anyhow::anyhow!("No .changelog directory found. Run 'changement init' first."));
    }

    info!("Reading changes from .changelog directory...");
    let changes = read_changelog_files(&changelog_dir)?;
    
    if changes.is_empty() {
        info!("No changes found in .changelog directory.");
        return Ok(());
    }

    info!("Discovering npm workspace packages...");
    let mut packages = discover_workspace_packages(&cwd)?;
    
    // Apply filter if provided
    if let Some(filter) = filter {
        let filter_packages: std::collections::HashSet<String> = filter
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        packages.retain(|name, _| filter_packages.contains(name));
    }

    info!("Calculating version bumps...");
    let version_bumps = calculate_version_bumps(&changes, &packages)?;
    
    if version_bumps.is_empty() {
        info!("No version bumps to apply.");
        return Ok(());
    }

    info!("Applying version bumps to packages...");
    apply_version_bumps(&cwd, &mut packages, &version_bumps)?;

    info!("Removing processed changelog files...");
    for change in changes {
        fs::remove_file(&change.file_path)?;
        debug!("Removed {}", change.file_path.display());
    }

    info!("Version command completed successfully!");
    Ok(())
}

fn read_changelog_files(changelog_dir: &Path) -> Result<Vec<ChangeEntry>> {
    let mut changes = Vec::new();
    
    for entry in fs::read_dir(changelog_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(change) = parse_changelog_file(&path)? {
                changes.push(change);
            }
        }
    }
    
    Ok(changes)
}

fn parse_changelog_file(file_path: &Path) -> Result<Option<ChangeEntry>> {
    let content = fs::read_to_string(file_path)?;
    
    // Parse frontmatter using regex
    let re = Regex::new(r"(?s)^---\n(.*?)\n---")?;
    
    if let Some(captures) = re.captures(&content) {
        let frontmatter = captures.get(1).unwrap().as_str();
        let mut packages = HashMap::new();
        
        for line in frontmatter.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse YAML-like format: 'package-name': bump_type
            if let Some((package_part, bump_part)) = line.split_once(':') {
                let package_name = package_part.trim().trim_matches('\'').trim_matches('"');
                let bump_type = bump_part.trim();
                
                if !package_name.is_empty() && !bump_type.is_empty() {
                    let bump = BumpType::from_str(bump_type)?;
                    packages.insert(package_name.to_string(), bump);
                }
            }
        }
        
        if !packages.is_empty() {
            return Ok(Some(ChangeEntry {
                packages,
                file_path: file_path.to_path_buf(),
            }));
        }
    }
    
    Ok(None)
}

fn discover_workspace_packages(cwd: &Path) -> Result<HashMap<String, PackageJson>> {
    let mut packages = HashMap::new();
    
    // First check if there's a root package.json
    let root_package_path = cwd.join("package.json");
    if root_package_path.exists() {
        let content = fs::read_to_string(&root_package_path)?;
        if let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&content) {
            // Check if this is a workspace root
            if package_json.get("workspaces").is_some() {
                // Find workspace packages
                find_workspace_packages(cwd, &mut packages)?;
            } else if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                // Single package project
                packages.insert(pkg.name.clone(), pkg);
            }
        }
    }
    
    // Always also look for packages in subdirectories
    find_workspace_packages(cwd, &mut packages)?;
    
    Ok(packages)
}

fn find_workspace_packages(cwd: &Path, packages: &mut HashMap<String, PackageJson>) -> Result<()> {
    // Look for package.json files in subdirectories
    for entry in fs::read_dir(cwd)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
            let package_path = path.join("package.json");
            if package_path.exists() {
                let content = fs::read_to_string(&package_path)?;
                if let Ok(package_json) = serde_json::from_str::<PackageJson>(&content) {
                    packages.insert(package_json.name.clone(), package_json);
                }
            }
        }
    }
    
    Ok(())
}

fn calculate_version_bumps(
    changes: &[ChangeEntry],
    packages: &HashMap<String, PackageJson>,
) -> Result<HashMap<String, BumpType>> {
    let mut version_bumps = HashMap::new();
    
    // First pass: collect direct bumps from changelog files
    for change in changes {
        for (package_name, bump_type) in &change.packages {
            if packages.contains_key(package_name) {
                let existing_bump = version_bumps.get(package_name);
                if existing_bump.is_none() || bump_type.is_greater_than(existing_bump.unwrap()) {
                    version_bumps.insert(package_name.clone(), bump_type.clone());
                }
            }
        }
    }
    
    // Second pass: handle transitive dependencies
    let mut iteration = 0;
    let mut changed = true;
    while changed && iteration < 100 { // Safety limit to prevent infinite loops
        iteration += 1;
        changed = false;
        let current_bumps = version_bumps.clone();
        
        for (package_name, package_json) in packages {
            // Check if any dependencies are being bumped
            for dep_name in package_json.dependencies.keys()
                .chain(package_json.dev_dependencies.keys())
                .chain(package_json.peer_dependencies.keys()) {
                
                if let Some(dep_bump) = current_bumps.get(dep_name) {
                    let existing_bump = version_bumps.get(package_name);
                    
                    // If this package isn't already being bumped, or the dependency bump is greater
                    if existing_bump.is_none() || dep_bump.is_greater_than(existing_bump.unwrap()) {
                        version_bumps.insert(package_name.clone(), dep_bump.clone());
                        changed = true;
                    }
                }
            }
        }
    }
    
    if iteration >= 100 {
        return Err(anyhow::anyhow!("Dependency calculation exceeded maximum iterations (possible circular dependency)"));
    }
    
    Ok(version_bumps)
}

fn apply_version_bumps(
    cwd: &Path,
    packages: &mut HashMap<String, PackageJson>,
    version_bumps: &HashMap<String, BumpType>,
) -> Result<()> {
    for (package_name, bump_type) in version_bumps {
        if let Some(package_json) = packages.get_mut(package_name) {
            let current_version = Version::parse(&package_json.version)?;
            let new_version = match bump_type {
                BumpType::Major => Version::new(current_version.major + 1, 0, 0),
                BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
                BumpType::Patch => Version::new(current_version.major, current_version.minor, current_version.patch + 1),
            };
            
            package_json.version = new_version.to_string();
            
            // Find and update the package.json file
            let package_path = find_package_json_path(cwd, package_name)?;
            let updated_content = serde_json::to_string_pretty(package_json)?;
            fs::write(&package_path, updated_content)?;
            
            info!("Updated {} from {} to {}", package_name, current_version, new_version);
        }
    }
    
    Ok(())
}

fn find_package_json_path(cwd: &Path, package_name: &str) -> Result<PathBuf> {
    // First check root directory
    let root_package_path = cwd.join("package.json");
    if root_package_path.exists() {
        let content = fs::read_to_string(&root_package_path)?;
        if let Ok(package_json) = serde_json::from_str::<PackageJson>(&content) {
            if package_json.name == package_name {
                return Ok(root_package_path);
            }
        }
    }
    
    // Then check subdirectories
    for entry in fs::read_dir(cwd)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
            let package_path = path.join("package.json");
            if package_path.exists() {
                let content = fs::read_to_string(&package_path)?;
                if let Ok(package_json) = serde_json::from_str::<PackageJson>(&content) {
                    if package_json.name == package_name {
                        return Ok(package_path);
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Could not find package.json for package: {}", package_name))
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
    fn test_version_command_with_no_changelog_dir() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        let cmd = Command::Version { filter: None };

        let result = process(cwd, &cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No .changelog directory found"));
    }

    #[test]
    fn test_version_command_with_empty_changelog() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Create .changelog directory
        fs::create_dir_all(temp_dir.path().join(".changelog")).unwrap();
        
        let cmd = Command::Version { filter: None };
        let result = process(cwd, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_command_full_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let cwd = temp_dir.path().to_path_buf();
        
        // Create .changelog directory
        fs::create_dir_all(temp_dir.path().join(".changelog")).unwrap();
        
        // Create a sample package.json
        let package_json = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "dependencies": {}
}"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        // Create a changelog file
        let changelog_content = r#"---
'test-package': minor
---

Added a new feature
"#;
        fs::write(temp_dir.path().join(".changelog/feature.md"), changelog_content).unwrap();
        
        let cmd = Command::Version { filter: None };
        let result = process(cwd, &cmd);
        assert!(result.is_ok());
        
        // Check that package.json was updated
        let updated_content = fs::read_to_string(temp_dir.path().join("package.json")).unwrap();
        let updated_package: serde_json::Value = serde_json::from_str(&updated_content).unwrap();
        assert_eq!(updated_package["version"], "1.1.0");
        
        // Check that changelog file was removed
        assert!(!temp_dir.path().join(".changelog/feature.md").exists());
    }
}
