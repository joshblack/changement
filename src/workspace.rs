use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a JavaScript workspace with a package.json file
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    /// The path to the workspace directory
    pub path: PathBuf,
    /// The parsed package.json content
    pub package_json: PackageJson,
    /// Child workspaces discovered from this workspace
    pub children: Vec<Workspace>,
}

/// Represents the contents of a package.json file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageJson {
    /// Package name
    pub name: Option<String>,
    /// Package version
    pub version: Option<String>,
    /// Workspace patterns (for npm/yarn workspaces)
    pub workspaces: Option<Vec<String>>,
    /// Private flag
    pub private: Option<bool>,
}

/// Represents the contents of a pnpm-workspace.yaml file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PnpmWorkspace {
    /// Package patterns
    pub packages: Vec<String>,
}

/// Represents a project containing JavaScript workspaces
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    /// The root path of the project
    pub root: PathBuf,
    /// All discovered workspaces in the project
    pub workspaces: Vec<Workspace>,
}

impl Project {
    /// Load a project and discover all workspaces starting from the given directory
    pub fn load<P: AsRef<Path>>(root: P) -> Result<Self, Box<dyn std::error::Error>> {
        let root = root.as_ref().to_path_buf();
        let workspaces = discover_workspaces(&root)?;

        Ok(Project { root, workspaces })
    }

    /// Get all workspace paths in the project
    #[allow(dead_code)]
    pub fn workspace_paths(&self) -> Vec<&PathBuf> {
        collect_workspace_paths(&self.workspaces)
    }
}

/// Discover all workspaces in a project starting from the root directory
fn discover_workspaces(root: &Path) -> Result<Vec<Workspace>, Box<dyn std::error::Error>> {
    let mut workspaces = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Find all package.json files in the directory tree
    let package_json_files = find_package_json_files(root)?;

    // For each package.json, create a workspace and discover its children
    for package_json_path in package_json_files {
        let workspace_dir = package_json_path.parent().unwrap();

        if visited.contains(workspace_dir) {
            continue;
        }
        visited.insert(workspace_dir.to_path_buf());

        let workspace = load_workspace(workspace_dir)?;
        workspaces.push(workspace);
    }

    Ok(workspaces)
}

/// Find all package.json files in a directory tree
fn find_package_json_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        let path = entry.path();

        if path.file_name().and_then(|n| n.to_str()) == Some("package.json") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

/// Load a workspace from a directory containing a package.json file
fn load_workspace(workspace_dir: &Path) -> Result<Workspace, Box<dyn std::error::Error>> {
    let package_json_path = workspace_dir.join("package.json");
    let package_json = load_package_json(&package_json_path)?;

    // Find child workspaces from package.json workspaces field
    let mut children = Vec::new();
    if let Some(workspace_patterns) = &package_json.workspaces {
        children.extend(find_workspaces_from_patterns(
            workspace_dir,
            workspace_patterns,
        )?);
    }

    // Find child workspaces from pnpm-workspace.yaml
    let pnpm_workspace_path = workspace_dir.join("pnpm-workspace.yaml");
    if pnpm_workspace_path.exists() {
        let pnpm_workspace = load_pnpm_workspace(&pnpm_workspace_path)?;
        children.extend(find_workspaces_from_patterns(
            workspace_dir,
            &pnpm_workspace.packages,
        )?);
    }

    Ok(Workspace {
        path: workspace_dir.to_path_buf(),
        package_json,
        children,
    })
}

/// Load and parse a package.json file
fn load_package_json(path: &Path) -> Result<PackageJson, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let package_json: PackageJson = serde_json::from_str(&content)?;
    Ok(package_json)
}

/// Load and parse a pnpm-workspace.yaml file
fn load_pnpm_workspace(path: &Path) -> Result<PnpmWorkspace, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let pnpm_workspace: PnpmWorkspace = serde_yaml::from_str(&content)?;
    Ok(pnpm_workspace)
}

/// Find workspaces from glob patterns
fn find_workspaces_from_patterns(
    base_dir: &Path,
    patterns: &[String],
) -> Result<Vec<Workspace>, Box<dyn std::error::Error>> {
    let mut workspaces = Vec::new();
    let mut visited = std::collections::HashSet::new();

    for pattern in patterns {
        let full_pattern = base_dir.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        for entry in glob(&pattern_str)? {
            let workspace_path = entry?;

            if visited.contains(&workspace_path) {
                continue;
            }
            visited.insert(workspace_path.clone());

            // Check if this directory contains a package.json
            let package_json_path = workspace_path.join("package.json");
            if package_json_path.exists() {
                let workspace = load_workspace(&workspace_path)?;
                workspaces.push(workspace);
            }
        }
    }

    Ok(workspaces)
}

/// Recursively collect all workspace paths
#[allow(dead_code)]
fn collect_workspace_paths(workspaces: &[Workspace]) -> Vec<&PathBuf> {
    let mut paths = Vec::new();
    for workspace in workspaces {
        paths.push(&workspace.path);
        paths.extend(collect_workspace_paths(&workspace.children));
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_simple_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = temp_dir.path().join("package.json");

        let package_json_content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "private": true
        }"#;

        fs::write(&package_json_path, package_json_content).unwrap();

        let package_json = load_package_json(&package_json_path).unwrap();

        assert_eq!(package_json.name, Some("test-package".to_string()));
        assert_eq!(package_json.version, Some("1.0.0".to_string()));
        assert_eq!(package_json.private, Some(true));
        assert_eq!(package_json.workspaces, None);
    }

    #[test]
    fn test_load_package_json_with_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = temp_dir.path().join("package.json");

        let package_json_content = r#"{
            "name": "monorepo",
            "version": "1.0.0",
            "private": true,
            "workspaces": ["packages/*", "apps/*"]
        }"#;

        fs::write(&package_json_path, package_json_content).unwrap();

        let package_json = load_package_json(&package_json_path).unwrap();

        assert_eq!(package_json.name, Some("monorepo".to_string()));
        assert_eq!(
            package_json.workspaces,
            Some(vec!["packages/*".to_string(), "apps/*".to_string()])
        );
    }

    #[test]
    fn test_load_pnpm_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let pnpm_workspace_path = temp_dir.path().join("pnpm-workspace.yaml");

        let pnpm_workspace_content = r#"packages:
  - "packages/*"
  - "apps/*"
"#;

        fs::write(&pnpm_workspace_path, pnpm_workspace_content).unwrap();

        let pnpm_workspace = load_pnpm_workspace(&pnpm_workspace_path).unwrap();

        assert_eq!(
            pnpm_workspace.packages,
            vec!["packages/*".to_string(), "apps/*".to_string()]
        );
    }

    #[test]
    fn test_find_package_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a structure with multiple package.json files
        fs::create_dir_all(root.join("packages/package-a")).unwrap();
        fs::create_dir_all(root.join("packages/package-b")).unwrap();
        fs::create_dir_all(root.join("apps/app-1")).unwrap();

        fs::write(root.join("package.json"), r#"{"name": "root"}"#).unwrap();
        fs::write(
            root.join("packages/package-a/package.json"),
            r#"{"name": "package-a"}"#,
        )
        .unwrap();
        fs::write(
            root.join("packages/package-b/package.json"),
            r#"{"name": "package-b"}"#,
        )
        .unwrap();
        fs::write(root.join("apps/app-1/package.json"), r#"{"name": "app-1"}"#).unwrap();

        let files = find_package_json_files(root).unwrap();

        assert_eq!(files.len(), 4);
        assert!(files.iter().any(|f| f.ends_with("package.json")));
        assert!(
            files
                .iter()
                .any(|f| f.ends_with("packages/package-a/package.json"))
        );
        assert!(
            files
                .iter()
                .any(|f| f.ends_with("packages/package-b/package.json"))
        );
        assert!(files.iter().any(|f| f.ends_with("apps/app-1/package.json")));
    }

    #[test]
    fn test_project_load_simple() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a simple project with one package.json
        fs::write(
            root.join("package.json"),
            r#"{"name": "simple-project", "version": "1.0.0"}"#,
        )
        .unwrap();

        let project = Project::load(root).unwrap();

        assert_eq!(project.root, root);
        assert_eq!(project.workspaces.len(), 1);
        assert_eq!(
            project.workspaces[0].package_json.name,
            Some("simple-project".to_string())
        );
    }

    #[test]
    fn test_project_load_with_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a monorepo structure
        fs::create_dir_all(root.join("packages/package-a")).unwrap();
        fs::create_dir_all(root.join("packages/package-b")).unwrap();

        fs::write(
            root.join("package.json"),
            r#"{
            "name": "monorepo",
            "version": "1.0.0",
            "private": true,
            "workspaces": ["packages/*"]
        }"#,
        )
        .unwrap();

        fs::write(
            root.join("packages/package-a/package.json"),
            r#"{"name": "package-a", "version": "1.0.0"}"#,
        )
        .unwrap();
        fs::write(
            root.join("packages/package-b/package.json"),
            r#"{"name": "package-b", "version": "1.0.0"}"#,
        )
        .unwrap();

        let project = Project::load(root).unwrap();

        assert_eq!(project.workspaces.len(), 3); // root + 2 packages

        let workspace_names: Vec<_> = project
            .workspaces
            .iter()
            .filter_map(|w| w.package_json.name.as_ref())
            .collect();

        assert!(workspace_names.contains(&&"monorepo".to_string()));
        assert!(workspace_names.contains(&&"package-a".to_string()));
        assert!(workspace_names.contains(&&"package-b".to_string()));
    }
}
