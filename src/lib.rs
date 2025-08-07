#![deny(clippy::all)]

#[cfg(feature = "napi")]
use napi_derive::napi;

/// Core functionality that doesn't depend on napi
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Main changement function that handles CLI operations
pub fn changement_main(args: Vec<String>) -> String {
    // For now, return a simple message
    // This would be where the actual CLI logic goes
    if args.is_empty() {
        return "changement - Manage versioning and publishing for packages in your project"
            .to_string();
    }

    match args[0].as_str() {
    "version" => get_version(),
    "help" | "--help" | "-h" => {
      "changement - Manage versioning and publishing for packages in your project\n\nCommands:\n  init     Initialize changement in a new project\n  new      Create a new change for a package\n  version  Apply all changes and update package versions\n  publish  Publish packages to registry\n  tag      Create git tags for current versions".to_string()
    }
    _ => format!("Unknown command: {}. Use 'changement help' for available commands.", args[0])
  }
}

/// Initialize changement in a project
pub fn init() -> String {
    "Initializing changement...".to_string()
}

/// Create a new change
pub fn new_change(package: String, message: String, bump: String) -> String {
    format!("Creating new {bump} change for package '{package}': {message}")
}

// NAPI bindings (only compiled when napi feature is enabled)
#[cfg(feature = "napi")]
#[napi]
pub fn get_version_napi() -> String {
    get_version()
}

#[cfg(feature = "napi")]
#[napi]
pub fn changement_main_napi(args: Vec<String>) -> String {
    changement_main(args)
}

#[cfg(feature = "napi")]
#[napi]
pub fn init_napi() -> String {
    init()
}

#[cfg(feature = "napi")]
#[napi]
pub fn new_change_napi(package: String, message: String, bump: String) -> String {
    new_change(package, message, bump)
}
