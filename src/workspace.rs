use anyhow::{Context, Result};
use ignore::Walk;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Workspace {
    directory: PathBuf,
    package_json: Result<PackageJson>,
}

impl Workspace {
    fn new(directory: PathBuf, package_json: Result<PackageJson>) -> Self {
        Self {
            directory,
            package_json,
        }
    }
}

pub fn list_workspaces(directory: PathBuf) -> Vec<Workspace> {
    let mut package_json_paths: Vec<PathBuf> = Vec::new();
    let walker = Walk::new(directory).into_iter();

    for entry in walker {
        if let Ok(entry) = entry
            && entry.path().ends_with("package.json")
        {
            package_json_paths.push(entry.into_path());
        }
    }

    package_json_paths
        .iter()
        .filter_map(|package_json_path| {
            let contents = fs::read_to_string(package_json_path);
            if let Ok(contents) = contents {
                let package_json = serde_json::from_str::<PackageJson>(&contents)
                    .context("Unable to parse package.json");
                Some(Workspace::new(
                    package_json_path.parent().unwrap().to_path_buf(),
                    package_json,
                ));
            }

            None
        })
        .collect::<Vec<Workspace>>()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageJson {
    name: Option<String>,
    version: Option<String>,

    #[serde(default)]
    workspaces: Vec<String>,

    #[serde(default)]
    dependencies: HashMap<String, String>,

    #[serde(rename = "devDependencies", default)]
    dev_dependencies: HashMap<String, String>,

    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_get_single_workspace() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("test");
        let package_json_path = workspace_dir.join("package.json");
        let package_json = PackageJson {
            name: None,
            version: None,
            workspaces: Default::default(),
            dependencies: Default::default(),
            dev_dependencies: Default::default(),
            peer_dependencies: Default::default(),
        };

        fs::create_dir(&workspace_dir)?;
        fs::write(&package_json_path, serde_json::to_string(&package_json)?)?;

        let ws = list_workspaces(workspace_dir);

        println!("{:?}", ws);

        Ok(())
    }
}
