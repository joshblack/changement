use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

pub fn list_workspaces(directory: PathBuf) {
    //
}

pub struct Workspace {
    directory: PathBuf,
    name: Option<String>,
    version: Option<String>,
    dependencies: Vec<(String, String)>,
}

impl Workspace {
    pub fn new(directory: PathBuf, package_json: PackageJson) -> Self {
        let mut dependencies: Vec<(String, String)> = Vec::new();

        for (name, version) in package_json.dependencies {
            dependencies.push((name, version));
        }

        for (name, version) in package_json.dev_dependencies {
            dependencies.push((name, version));
        }

        for (name, version) in package_json.peer_dependencies {
            dependencies.push((name, version));
        }

        Self {
            directory,
            name: package_json.name,
            version: package_json.version,
            dependencies,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
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

    #[test]
    fn test_workspace_new() {
        let package_json = PackageJson {
            name: Some("test-package".to_string()),
            version: Some("1.0.0".to_string()),
            workspaces: vec![],
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
        };
        let workspace = Workspace::new(PathBuf::from("/test/path"), package_json);
        assert_eq!(workspace.name, Some("test-package".to_string()));
        assert_eq!(workspace.version, Some("1.0.0".to_string()));
        assert!(workspace.dependencies.is_empty());
    }

    #[test]
    fn test_workspace_dependencies() {
        let package_json = PackageJson {
            name: Some("test-package".to_string()),
            version: Some("1.0.0".to_string()),
            workspaces: vec![],
            dependencies: HashMap::from([("dep1".into(), "1.0.0".into())]),
            dev_dependencies: HashMap::from([("dep2".into(), "1.0.0".into())]),
            peer_dependencies: HashMap::from([("dep3".into(), "1.0.0".into())]),
        };
        let workspace = Workspace::new(PathBuf::from("/test/path"), package_json);
        assert_eq!(
            workspace.dependencies,
            Vec::from([
                ("dep1".to_string(), "1.0.0".to_string()),
                ("dep2".to_string(), "1.0.0".to_string()),
                ("dep3".to_string(), "1.0.0".to_string()),
            ])
        );
    }

    #[test]
    fn test_workspace_dependency_overlap() {
        let package_json = PackageJson {
            name: Some("test-package".to_string()),
            version: Some("1.0.0".to_string()),
            workspaces: vec![],
            dependencies: HashMap::from([("dep".into(), "1.0.0".into())]),
            dev_dependencies: HashMap::from([("dep".into(), "2.0.0".into())]),
            peer_dependencies: HashMap::new(),
        };
        let workspace = Workspace::new(PathBuf::from("/test/path"), package_json);
        assert_eq!(
            workspace.dependencies,
            Vec::from([
                ("dep".to_string(), "1.0.0".to_string()),
                ("dep".to_string(), "2.0.0".to_string()),
            ])
        );
    }
}
