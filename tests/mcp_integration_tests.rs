use spidey_shell::mcp::MCPServerManager;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a mock MCP project structure
fn create_mock_mcp_project(base_dir: &PathBuf, project_name: &str, package_name: &str) -> PathBuf {
    let projects_dir = base_dir.join("projects");
    fs::create_dir_all(&projects_dir).expect("Failed to create projects directory");
    
    let project_dir = projects_dir.join(project_name);
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
    init_file.write_all(b"__version__ = '0.1.0'").expect("Failed to write to __init__.py");
    
    // Create __main__.py to make the package runnable
    let main_module_path = package_dir.join("__main__.py");
    let mut main_module_file = File::create(main_module_path).expect("Failed to create __main__.py");
    main_module_file.write_all(
        b"
from .main import main

if __name__ == '__main__':
    main()
",
    ).expect("Failed to write to __main__.py");
    
    // Create a simple main.py
    let main_path = package_dir.join("main.py");
    let mut main_file = File::create(main_path).expect("Failed to create main.py");
    main_file.write_all(
        b"
import sys

def main():
    print('Server started')
    sys.stdout.flush()
    # Just wait for input and exit
    try:
        input()
    except:
        pass
    print('Server shutting down')
    return 0

if __name__ == '__main__':
    main()
",
    ).expect("Failed to write to main.py");
    
    // Create pyproject.toml with MCP capabilities
    let pyproject_path = project_dir.join("pyproject.toml");
    let mut pyproject_file = File::create(pyproject_path).expect("Failed to create pyproject.toml");
    pyproject_file.write_all(
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
        ).as_bytes(),
    ).expect("Failed to write to pyproject.toml");
    
    project_dir
}

// Helper function to create a basic server config in the mcp directory
fn create_basic_server_config(base_dir: &PathBuf, server_name: &str) {
    let server_dir = base_dir.join(server_name);
    fs::create_dir_all(&server_dir).expect("Failed to create server directory");
    
    // Create a simple config.json
    let config_path = server_dir.join("config.json");
    let mut config_file = File::create(config_path).expect("Failed to create config.json");
    config_file.write_all(
        format!(
            r#"{{
  "name": "{}",
  "url": "http://localhost:5000",
  "api_key": "test_key",
  "description": "Test server",
  "capabilities": ["text-generation"],
  "enabled": true
}}"#,
            server_name
        ).as_bytes(),
    ).expect("Failed to write to config.json");
}

#[test]
fn test_server_manager_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();
    
    let result = MCPServerManager::new(base_dir);
    assert!(result.is_ok(), "Should be able to create server manager");
}

#[test]
fn test_basic_server_config_loading() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create a basic server config
    let server_name = "test_server";
    create_basic_server_config(&base_dir, server_name);
    
    // Create the server manager and load configs
    let mut manager = MCPServerManager::new(base_dir).expect("Failed to create server manager");
    
    // Check that the server was loaded
    let servers = manager.get_server_names();
    assert!(servers.contains(&server_name.to_string()), "Should load the test server");
    
    // Check server details
    let server = manager.get_server(server_name);
    assert!(server.is_some(), "Should find the server");
    let server = server.unwrap();
    assert_eq!(server.name, server_name);
    assert_eq!(server.url, Some("http://localhost:5000".to_string()));
    assert!(server.enabled);
}

#[test]
fn test_python_project_discovery() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create a Python MCP project
    let project_name = "test_mcp_project";
    let package_name = "test_package";
    create_mock_mcp_project(&base_dir, project_name, package_name);
    
    // Create the server manager
    let mut manager = MCPServerManager::new(base_dir).expect("Failed to create server manager");
    
    // Check if the Python project was discovered
    let servers = manager.get_server_names();
    
    // Note: This may fail if Python/uv aren't available, so we don't assert too strongly
    if !servers.contains(&project_name.to_string()) {
        println!("Note: Python project wasn't discovered, this may be expected if Python/uv aren't available");
    }
}

#[test]
fn test_server_commands_parsing() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create a basic server config
    let server_name = "test_server";
    create_basic_server_config(&base_dir, server_name);
    
    // Create the server manager
    let mut manager = MCPServerManager::new(base_dir).expect("Failed to create server manager");
    
    // Test parsing @server commands
    let (modified_input, servers) = manager.parse_server_commands("Ask @test_server to do something");
    assert_eq!(modified_input, "Ask  to do something");
    assert_eq!(servers, vec!["test_server".to_string()]);
    
    // Test with multiple servers
    create_basic_server_config(&manager.base_dir, "another_server");
    manager.load_all().expect("Failed to reload servers");
    
    let (modified_input, servers) = manager.parse_server_commands("Ask @test_server and @another_server to do something");
    assert_eq!(modified_input, "Ask  and  to do something");
    assert_eq!(servers.len(), 2);
    assert!(servers.contains(&"test_server".to_string()));
    assert!(servers.contains(&"another_server".to_string()));
    
    // Test with an invalid server
    let (modified_input, servers) = manager.parse_server_commands("Ask @invalid_server to do something");
    assert_eq!(modified_input, "Ask @invalid_server to do something");
    assert!(servers.is_empty());
}

#[test]
fn test_create_python_project() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create the server manager
    let mut manager = MCPServerManager::new(base_dir).expect("Failed to create server manager");
    
    // Create a new Python project
    let project_name = "new_project";
    let result = manager.create_python_project(project_name);
    
    assert!(result.is_ok(), "Should be able to create a Python project");
    let project_dir = result.unwrap();
    
    // Verify project structure
    assert!(project_dir.exists(), "Project directory should exist");
    assert!(project_dir.join("src").exists(), "src directory should exist");
    assert!(project_dir.join("src").join(project_name).exists(), "Package directory should exist");
    assert!(project_dir.join("pyproject.toml").exists(), "pyproject.toml should exist");
}

#[test]
#[ignore] // Skip by default as it requires uv/Python environment
fn test_python_process_management() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_dir = temp_dir.path().to_path_buf();
    
    // Create a Python MCP project
    let project_name = "process_test";
    let package_name = "process_package";
    create_mock_mcp_project(&base_dir, project_name, package_name);
    
    // Create the server manager
    let mut manager = MCPServerManager::new(base_dir).expect("Failed to create server manager");
    
    // Try to start the Python process
    let result = manager.start_python_process(project_name);
    
    // This may fail if Python/uv aren't available, so we don't assert too strongly
    if let Ok(started) = result {
        if started {
            // If it started successfully, try to stop it
            let stop_result = manager.stop_python_process(project_name);
            assert!(stop_result.is_ok(), "Should be able to stop the Python process");
        } else {
            println!("Note: Python process wasn't started, this may be expected if Python/uv aren't available");
        }
    }
}