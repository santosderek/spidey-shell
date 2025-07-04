use spidey_shell::mcp::python::{MCPProcessStatus, MCPPythonConfig, MCPPythonProcess};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a mock Python script that simulates an MCP server
fn create_mock_python_script(dir_path: &PathBuf, package_name: &str) {
    // Create package structure
    let src_dir = dir_path.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    let package_dir = src_dir.join(package_name);
    fs::create_dir_all(&package_dir).expect("Failed to create package directory");

    // Create __init__.py
    let init_path = package_dir.join("__init__.py");
    let mut init_file = File::create(init_path).expect("Failed to create __init__.py");
    init_file
        .write_all(b"")
        .expect("Failed to write to __init__.py");

    // Create a simple script that prints "Server started" and then waits
    let main_path = package_dir.join("main.py");
    let mut main_file = File::create(main_path).expect("Failed to create main.py");
    main_file
        .write_all(
            b"
import sys
import time

def main():
    print('Server started')
    sys.stdout.flush()  # Ensure output is flushed
    try:
        # Wait for any input to shut down
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print('Server shutting down')
    return 0

if __name__ == '__main__':
    sys.exit(main())
",
        )
        .expect("Failed to write to main.py");

    // Create __main__.py to make the package runnable
    let main_module_path = package_dir.join("__main__.py");
    let mut main_module_file =
        File::create(main_module_path).expect("Failed to create __main__.py");
    main_module_file
        .write_all(
            b"
from .main import main

if __name__ == '__main__':
    main()
",
        )
        .expect("Failed to write to __main__.py");

    // Create pyproject.toml
    let pyproject_path = dir_path.join("pyproject.toml");
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
description = "Test MCP Server"
requires-python = ">=3.8"

[tool.setuptools]
package-dir = {{ "" = "src" }}

[tool.setuptools.packages.find]
where = ["src"]
"#,
                package_name
            )
            .as_bytes(),
        )
        .expect("Failed to write to pyproject.toml");
}

#[test]
fn test_python_config_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    let config = MCPPythonConfig::new("test_server", project_path.clone(), "test_package");

    assert_eq!(config.name, "test_server");
    assert_eq!(config.project_path, project_path);
    assert_eq!(config.package_name, "test_package");
    assert_eq!(config.args.len(), 0);
    assert_eq!(config.env.len(), 0);
    assert_eq!(config.startup_timeout, 30);
    assert_eq!(config.port, None);
}

#[test]
fn test_python_config_customization() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    let mut config = MCPPythonConfig::new("test_server", project_path, "test_package");

    // Customize the configuration
    config.args = vec!["--debug".to_string()];
    let mut env = HashMap::new();
    env.insert("DEBUG".to_string(), "1".to_string());
    config.env = env;
    config.startup_timeout = 60;
    config.port = Some(8080);

    assert_eq!(config.args, vec!["--debug"]);
    assert_eq!(config.env.get("DEBUG").unwrap(), "1");
    assert_eq!(config.startup_timeout, 60);
    assert_eq!(config.port, Some(8080));
}

// This test requires Python and uv to be installed
#[test]
#[ignore] // Skip by default as it requires uv/Python environment
fn test_python_process_start_stop() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let package_name = "test_mock_server";

    // Create a mock Python script
    create_mock_python_script(&project_path, package_name);

    // Create a Python process config
    let config = MCPPythonConfig::new("test_server", project_path, package_name);
    let mut process = MCPPythonProcess::new(config);

    // Test initial state
    assert_eq!(process.status(), MCPProcessStatus::NotRunning);
    assert!(!process.is_running());

    // Start the process (this requires uv and Python to be installed)
    match process.start() {
        Ok(_) => {
            // Wait a bit for the process to start
            std::thread::sleep(std::time::Duration::from_secs(2));

            // Check if process is running
            assert!(
                process.status() == MCPProcessStatus::Running
                    || process.status() == MCPProcessStatus::Starting
            );

            // Stop the process
            process.stop().expect("Failed to stop process");

            // Check if process is stopped
            assert_eq!(process.status(), MCPProcessStatus::NotRunning);
        }
        Err(e) => {
            panic!("Failed to start Python process: {}", e);
        }
    }
}

// This test will mock the process to avoid actual Python dependency
#[test]
fn test_process_status_transitions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();

    let config = MCPPythonConfig::new("test_server", project_path, "test_package");
    let process = MCPPythonProcess::new(config);

    // Test initial state
    assert_eq!(process.status(), MCPProcessStatus::NotRunning);

    // We can't easily test the actual process state transitions without
    // running a real Python process, but we can at least verify the status types
    assert_ne!(MCPProcessStatus::NotRunning, MCPProcessStatus::Starting);
    assert_ne!(MCPProcessStatus::NotRunning, MCPProcessStatus::Running);
    assert_ne!(MCPProcessStatus::NotRunning, MCPProcessStatus::ShuttingDown);
    assert_ne!(MCPProcessStatus::NotRunning, MCPProcessStatus::Failed);
}
