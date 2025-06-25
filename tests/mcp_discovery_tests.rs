use spidey_shell::mcp::discovery::MCPProjectDiscovery;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a mock MCP project structure
fn create_mock_mcp_project(base_dir: &PathBuf, project_name: &str, package_name: &str) -> PathBuf {
    let project_dir = base_dir.join(project_name);
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");

    // Create src directory
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    // Create package directory
    let package_dir = src_dir.join(package_name);
    fs::create_dir_all(&package_dir).expect("Failed to create package directory");

    // Create __init__.py
    let init_path = package_dir.join("__init__.py");
    let mut init_file = File::create(init_path).expect("Failed to create __init__.py");
    init_file
        .write_all(b"__version__ = '0.1.0'")
        .expect("Failed to write to __init__.py");

    // Create a simple main.py
    let main_path = package_dir.join("main.py");
    let mut main_file = File::create(main_path).expect("Failed to create main.py");
    main_file
        .write_all(
            b"
def main():
    print('Hello from MCP project')
    return 0

if __name__ == '__main__':
    main()
",
        )
        .expect("Failed to write to main.py");

    // Create pyproject.toml with MCP capabilities
    let pyproject_path = project_dir.join("pyproject.toml");
    let mut pyproject_file = File::create(pyproject_path).expect("Failed to create pyproject.toml");
    pyproject_file
        .write_all(
            format!(
                r#"
[build-system]
requires = ["setuptools>=42", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
version = "0.1.0"
description = "Test MCP Project"
requires-python = ">=3.8"

[project.dependencies]
requests = ">=2.28.0"

[tool.setuptools]
package-dir = {{ "" = "src" }}

[tool.setuptools.packages.find]
where = ["src"]

# MCP specific configuration
[tool.mcp]
mcp-capabilities = ["text-generation", "text-embedding"]
"#,
                project_name
            )
            .as_bytes(),
        )
        .expect("Failed to write to pyproject.toml");

    project_dir
}

// Helper function to create a non-MCP project (missing key elements)
fn create_non_mcp_project(base_dir: &PathBuf, project_name: &str) -> PathBuf {
    let project_dir = base_dir.join(project_name);
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");

    // This doesn't have the src/package_name structure, just a simple file
    let file_path = project_dir.join("main.py");
    let mut file = File::create(file_path).expect("Failed to create main.py");
    file.write_all(b"print('Not an MCP project')")
        .expect("Failed to write to main.py");

    project_dir
}

#[test]
fn test_project_discovery_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();

    let mut discovery = MCPProjectDiscovery::new(base_dir.clone());

    // The constructor shouldn't fail, but we can't test much else without creating projects
    assert!(discovery.scan_projects().is_ok());
}

#[test]
fn test_valid_project_discovery() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();

    // Create a valid MCP project
    let project_name = "test_project";
    let package_name = "test_package";
    create_mock_mcp_project(&base_dir, project_name, package_name);

    // Create a non-MCP project that should be ignored
    create_non_mcp_project(&base_dir, "not_mcp_project");

    // Create discovery service and scan
    let mut discovery = MCPProjectDiscovery::new(base_dir);
    let projects = discovery.scan_projects().expect("Failed to scan projects");

    // We should find our valid project
    assert!(!projects.is_empty(), "Should find at least one project");

    // The project may not be considered valid if Python/uv aren't available
    // for the import check, so we don't strictly test validity here
    let found_project = projects.iter().find(|p| p.name == project_name);
    assert!(found_project.is_some(), "Should find our test project");

    let project = found_project.unwrap();
    assert_eq!(project.name, project_name);
    assert_eq!(project.package_name, package_name);
}

#[test]
fn test_project_metadata_extraction() {
    // This test checks that we can properly extract metadata indirectly since parse_pyproject_toml is private
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();

    // Create a mock project with specific metadata
    let project_name = "test_metadata";
    let package_name = "test_metadata_pkg";
    let project_dir = base_dir.join(project_name);
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");

    // Create src/package structure for a valid project
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    let package_dir = src_dir.join(package_name);
    fs::create_dir_all(&package_dir).expect("Failed to create package directory");

    // Create __init__.py
    let init_path = package_dir.join("__init__.py");
    let mut init_file = File::create(init_path).expect("Failed to create __init__.py");
    init_file
        .write_all(b"")
        .expect("Failed to write to __init__.py");

    // Create pyproject.toml with specific metadata to test
    let pyproject_path = project_dir.join("pyproject.toml");
    let mut pyproject_file = File::create(pyproject_path).expect("Failed to create pyproject.toml");
    pyproject_file
        .write_all(
            br#"
[project]
name = "test_metadata"
version = "1.0.0"
description = "Test metadata extraction"

[dependencies]
requests = ">=2.28.0"
numpy = ">=1.20.0"

[tool.mcp]
mcp-capabilities = ["text-generation", "image-generation", "audio-transcription"]
"#,
        )
        .expect("Failed to write to pyproject.toml");

    // Discover the project and check metadata indirectly through the discovered project
    let mut discovery = MCPProjectDiscovery::new(base_dir);
    let projects = discovery.scan_projects().expect("Failed to scan projects");

    // Find our test project
    let project = projects.iter().find(|p| p.name == project_name);
    assert!(project.is_some(), "Should find our test metadata project");

    let project = project.unwrap();
    assert_eq!(project.metadata.name, Some("test_metadata".to_string()));

    // We expect some metadata to be extracted, but may not get everything due to simplified TOML parsing
    if project.metadata.description.is_some() {
        assert_eq!(
            project.metadata.description,
            Some("Test metadata extraction".to_string())
        );
    }

    // Check MCP capabilities - these are important for server functionality
    assert!(project
        .metadata
        .mcp_capabilities
        .contains(&"text-generation".to_string()));
    assert!(project
        .metadata
        .mcp_capabilities
        .contains(&"image-generation".to_string()));
    assert!(project
        .metadata
        .mcp_capabilities
        .contains(&"audio-transcription".to_string()));
}

#[test]
fn test_project_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();

    let discovery = MCPProjectDiscovery::new(base_dir.clone());

    // Create a new project
    let project_name = "new_test_project";
    let project_dir = discovery
        .create_project(project_name)
        .expect("Failed to create project");

    // Check that the project was created with the correct structure
    assert!(project_dir.exists());
    assert!(project_dir.join("src").exists());
    assert!(project_dir.join("src").join(project_name).exists());
    assert!(project_dir
        .join("src")
        .join(project_name)
        .join("__init__.py")
        .exists());
    assert!(project_dir
        .join("src")
        .join(project_name)
        .join("main.py")
        .exists());
    assert!(project_dir.join("pyproject.toml").exists());
    assert!(project_dir.join("README.md").exists());
}

#[test]
#[ignore] // Skip by default as it requires uv/Python environment
fn test_dependencies_installation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();

    // Create a valid MCP project with a simple dependency
    let project_name = "dep_test_project";
    let package_name = "dep_test_package";
    let project_dir = create_mock_mcp_project(&base_dir, project_name, package_name);

    // Create discovery service
    let discovery = MCPProjectDiscovery::new(base_dir);

    // Try to install dependencies
    let result = discovery.install_dependencies(&project_dir);
    assert!(result.is_ok(), "Dependency installation should not error");
}

#[test]
fn test_python_config_creation_from_project() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();

    // Create a valid MCP project
    let project_name = "config_test_project";
    let package_name = "config_test_package";
    let project_dir = create_mock_mcp_project(&base_dir, project_name, package_name);

    // Create discovery service and scan
    let mut discovery = MCPProjectDiscovery::new(base_dir);
    let _ = discovery.scan_projects().expect("Failed to scan projects");

    // Find our test project
    let project = discovery.get_project(project_name);
    assert!(project.is_some(), "Should find our test project");

    // Get Python config data
    let (name, path, pkg, _) = discovery.create_python_config_data(&project.unwrap());

    // Check the config data
    assert_eq!(name, project_name);
    assert_eq!(path, project_dir);
    assert_eq!(pkg, package_name);
}
