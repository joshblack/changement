use crate::graph::{Graph, NodeData, NodeIndex};
use globset::{Glob, GlobSetBuilder};
use ignore::Walk;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

pub struct Project {
    directory: PathBuf,
    graph: Graph<Workspace>,
}

impl Project {
    pub fn new(directory: impl AsRef<Path>) -> Self {
        let mut workspaces: Vec<(NodeIndex, Vec<String>)> = Vec::new();
        let mut graph = Graph::new();
        let walker = Walk::new(&directory);

        for result in walker.filter_map(|e| e.ok()) {
            if let Some(file_type) = result.file_type()
                && file_type.is_file()
                && result.file_name() == "package.json"
            {
                let package_json_path = result.path();
                let content = std::fs::read_to_string(package_json_path)
                    .expect("Failed to read package.json file");
                let package_json = serde_json::from_str::<PackageJson>(&content)
                    .expect("Failed to parse package.json");
                let workspace = Workspace::new(package_json_path, &package_json);
                let node_index = graph.add_node(workspace);
                workspaces.push((node_index, package_json.workspaces));
            }
        }

        for (node_index, child_workspace_patterns) in &workspaces {
            let workspace = graph
                .get_node(*node_index)
                .expect("Node index should exist in the graph");
            let mut builder = GlobSetBuilder::new();

            for pattern in child_workspace_patterns {
                let glob = Glob::new(&pattern).expect("Invalid glob pattern");
                builder.add(glob);
            }

            let set = builder.build().expect("Failed to build GlobSet");
            let walker = Walk::new(&workspace.directory)
                .filter_map(|e| e.ok())
                .filter(|e| set.is_match(e.path()))
                .filter(|e| {
                    if let Some(file_type) = e.file_type()
                        && file_type.is_dir()
                        && e.path().join("package.json").exists()
                    {
                        true
                    } else {
                        false
                    }
                });

            for directory in walker {
                let child_workspace = &workspaces.iter().find_map(|(child_workspace_index, _)| {
                    let child_workspace = graph
                        .get_node(*child_workspace_index)
                        .expect("Child workspace node should exist in the graph");

                    if child_workspace.directory == directory.path() {
                        Some(*child_workspace_index)
                    } else {
                        None
                    }
                });

                if let Some(child_workspace_index) = child_workspace {
                    graph.add_edge(*node_index, *child_workspace_index);
                }
            }
        }

        Self {
            directory: directory.as_ref().to_path_buf(),
            graph,
        }
    }

    pub fn get_workspaces(&self) -> impl Iterator<Item = &NodeData<Workspace>> {
        self.graph.get_nodes()
    }

    pub fn get_workspace(&self, name: &str) -> Option<&NodeData<Workspace>> {
        self.graph
            .get_nodes()
            .find(|node| node.data.name.as_deref() == Some(name))
    }

    pub fn dependents(&self, workspace: NodeIndex) -> impl Iterator<Item = &NodeData<Workspace>> {
        self.graph
            .edges(workspace)
            .filter_map(|node_index| self.graph.get_node(node_index))
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct Workspace {
    directory: PathBuf,
    name: Option<String>,
    version: Option<Version>,
    dependencies: Vec<(String, String)>,
}

impl Workspace {
    pub fn new(directory: impl AsRef<Path>, package_json: &PackageJson) -> Self {
        let mut dependencies: Vec<(String, String)> = Vec::new();

        for (name, version) in &package_json.dependencies {
            dependencies.push((name.into(), version.into()));
        }

        for (name, version) in &package_json.dev_dependencies {
            dependencies.push((name.into(), version.into()));
        }

        for (name, version) in &package_json.peer_dependencies {
            dependencies.push((name.into(), version.into()));
        }

        Self {
            directory: directory.as_ref().to_path_buf(),
            name: package_json.name.clone(),
            version: package_json.version.clone(),
            dependencies,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackageJson {
    name: Option<String>,
    version: Option<Version>,

    #[serde(default)]
    workspaces: Vec<String>,

    #[serde(default)]
    dependencies: HashMap<String, String>,

    #[serde(rename = "devDependencies", default)]
    dev_dependencies: HashMap<String, String>,

    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: HashMap<String, String>,
}

impl Default for PackageJson {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
            workspaces: vec![],
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_new() {
        let package_json = PackageJson {
            name: Some("test-package".to_string()),
            version: Some(Version::new(1, 0, 0)),
            workspaces: vec![],
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
        };
        let workspace = Workspace::new(PathBuf::from("/test/path"), &package_json);
        assert_eq!(workspace.name, Some("test-package".to_string()));
        assert_eq!(workspace.version, Some(Version::new(1, 0, 0)));
        assert!(workspace.dependencies.is_empty());
    }

    fn setup_single_workspace(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let package_json: PackageJson = Default::default();
        let package_json_path = path.join("package.json");

        std::fs::write(&package_json_path, serde_json::to_string(&package_json)?)?;

        Ok(())
    }

    #[test]
    fn test_project_new() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let directory = temp_dir.path().to_path_buf();

        setup_single_workspace(&directory)?;

        let project = Project::new(&directory);
        let workspaces = project.get_workspaces();

        assert_eq!(workspaces.count(), 1,);

        Ok(())
    }

    fn setup_multiple_workspaces(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let package_json = PackageJson {
            workspaces: vec!["packages/*".into()],
            ..Default::default()
        };
        let package_json_path = path.join("package.json");

        std::fs::write(&package_json_path, serde_json::to_string(&package_json)?)?;

        let packages = vec!["a", "b", "c"];

        for package in &packages {
            let package_path = path.join("packages").join(package);
            std::fs::create_dir_all(&package_path)?;
            let package_json_path = package_path.join("package.json");
            let package_json = PackageJson {
                name: Some(format!("{}", package)),
                ..Default::default()
            };
            std::fs::write(&package_json_path, serde_json::to_string(&package_json)?)?;
        }

        Ok(())
    }

    #[test]
    fn test_project_multiple_workspaces() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let directory = temp_dir.path().to_path_buf();

        setup_multiple_workspaces(&directory)?;

        let project = Project::new(&directory);
        let workspaces = project.get_workspaces();
        assert_eq!(workspaces.count(), 4);

        Ok(())
    }
}
