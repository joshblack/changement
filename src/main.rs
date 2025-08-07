use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "changement")]
#[command(about = "Manage versioning and publishing for packages in your project")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize changement in a new project
    Init,
    /// Show version information
    Version,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // If no arguments provided, show help
    if args.is_empty() {
        let result = changement::changement_main(vec![]);
        println!("{result}");
        return;
    }

    // Handle legacy direct argument passing for compatibility
    if let Some(first_arg) = args.first() {
        match first_arg.as_str() {
            "version" | "help" | "--help" | "-h" => {
                let result = changement::changement_main(args);
                println!("{result}");
                return;
            }
            _ => {}
        }
    }

    // Parse with clap for structured commands
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => {
            if let Err(e) = init_command() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Version) => {
            println!("{}", changement::get_version());
        }
        None => {
            let result = changement::changement_main(vec![]);
            println!("{result}");
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
    #[test]
    fn test_example() {
        assert_eq!(true, true);
    }
}
