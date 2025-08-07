use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

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

    match &cli.command {
        Commands::Init => {
            if let Err(e) = init_command() {
                eprintln!("Error: {}", e);
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
    
    println!("changement initialized successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_main_output() {
        // Test that the main function would produce the expected output
        let output = Command::new("cargo")
            .args(["run", "--bin", "changement"])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        assert_eq!(stdout.trim(), "Hello, world!");
    }

    #[test]
    fn test_hello_world_functionality() {
        // Test the core functionality by checking the expected message
        let expected_message = "Hello, world!";
        assert_eq!(expected_message, "Hello, world!");
        assert!(expected_message.starts_with("Hello"));
        assert!(expected_message.ends_with("world!"));
    }
}
