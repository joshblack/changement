use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

mod workspace;
use workspace::Project;

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
    /// List all workspaces in the current project
    Workspaces,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            if let Err(e) = init_command() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Workspaces => {
            if let Err(e) = workspaces_command() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn init_command() -> Result<(), Box<dyn std::error::Error>> {
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

    // Discover workspaces in the current project
    match Project::load(".") {
        Ok(project) => {
            println!(
                "Discovered {} workspace(s) in the project:",
                project.workspaces.len()
            );
            for workspace in &project.workspaces {
                let name = workspace.package_json.name.as_deref().unwrap_or("<unnamed>");
                println!("  - {name} ({})", workspace.path.display());
            }
        }
        Err(e) => {
            println!("Warning: Could not discover workspaces: {e}");
        }
    }

    println!("changement initialized successfully!");
    Ok(())
}

fn workspaces_command() -> Result<(), Box<dyn std::error::Error>> {
    let project = Project::load(".")?;

    println!("Found {} workspace(s):", project.workspaces.len());

    for workspace in &project.workspaces {
        let name = workspace.package_json.name.as_deref().unwrap_or("<unnamed>");
        let version = workspace.package_json.version.as_deref().unwrap_or("<no version>");
        let private = workspace.package_json.private.unwrap_or(false);

        println!();
        println!("📦 {name} ({version})");
        println!("   Path: {}", workspace.path.display());
        println!("   Private: {private}");

        if let Some(workspace_patterns) = &workspace.package_json.workspaces {
            if !workspace_patterns.is_empty() {
                println!("   Workspace patterns: {workspace_patterns:?}");
            }
        }

        if !workspace.children.is_empty() {
            println!("   Child workspaces: {}", workspace.children.len());
            for child in &workspace.children {
                let child_name = child.package_json.name.as_deref().unwrap_or("<unnamed>");
                println!("     - {child_name}");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example() {
        assert_eq!(true, true);
    }
}
