use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, warn};

// We're now in a hierarchical module structure, so we don't need forward declarations

/// Result of discovering an MCP Python project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPProjectInfo {
    /// Name of the project
    pub name: String,
    /// Path to the project directory
    pub project_path: PathBuf,
    /// Package name to run with python -m
    pub package_name: String,
    /// Path to the package directory
    pub package_path: PathBuf,
    /// Project metadata from pyproject.toml
    pub metadata: MCPProjectMetadata,
    /// Whether this project is currently valid
    pub is_valid: bool,
    /// Error message if the project is invalid
    pub error: Option<String>,
}

/// Metadata extracted from pyproject.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MCPProjectMetadata {
    /// Project name from pyproject.toml
    pub name: Option<String>,
    /// Project version from pyproject.toml
    pub version: Option<String>,
    /// Project description from pyproject.toml
    pub description: Option<String>,
    /// Project dependencies from pyproject.toml
    pub dependencies: Vec<String>,
    /// Supported MCP capabilities
    pub mcp_capabilities: Vec<String>,
}

/// Service to discover and manage MCP Python projects
pub struct MCPProjectDiscovery {
    /// Base directory to scan for projects
    base_dir: PathBuf,
    /// Cache of discovered projects
    projects: Vec<MCPProjectInfo>,
}

impl MCPProjectDiscovery {
    /// Create a new MCP project discovery service
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            projects: Vec::new(),
        }
    }

    /// Scan for MCP Python projects
    pub fn scan_projects(&mut self) -> io::Result<Vec<MCPProjectInfo>> {
        self.projects.clear();

        // Ensure the base directory exists
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir)?;
        }

        // Read directory entries
        let entries = fs::read_dir(&self.base_dir)?;

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if path.is_dir() {
                match self.validate_mcp_project(&path) {
                    Ok(Some(project_info)) => {
                        self.projects.push(project_info);
                    }
                    Ok(None) => {
                        // Not a valid MCP project, skip
                    }
                    Err(e) => {
                        warn!(
                            "Error validating potential MCP project at {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(self.projects.clone())
    }

    /// Validate if a directory contains a valid MCP Python project
    fn validate_mcp_project(&self, project_dir: &Path) -> io::Result<Option<MCPProjectInfo>> {
        let project_name = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid project directory name")
            })?
            .to_string();

        // Check for src directory
        let src_dir = project_dir.join("src");
        if !src_dir.is_dir() {
            debug!("Project {} has no src directory", project_name);
            return Ok(None);
        }

        // Check for pyproject.toml
        let pyproject_path = project_dir.join("pyproject.toml");
        if !pyproject_path.exists() {
            debug!("Project {} has no pyproject.toml", project_name);
            return Ok(None);
        }

        // Parse pyproject.toml to find the package name
        let metadata = match self.parse_pyproject_toml(&pyproject_path) {
            Ok(metadata) => metadata,
            Err(e) => {
                warn!("Failed to parse pyproject.toml for {}: {}", project_name, e);
                return Ok(None);
            }
        };

        // Determine the package name from the src directory
        // We're looking for a directory inside src/
        let mut package_name = String::new();
        let mut package_path = PathBuf::new();

        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let pkg_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Invalid package directory name")
                    })?
                    .to_string();

                // Check for __init__.py to confirm it's a Python package
                if path.join("__init__.py").exists() {
                    package_name = pkg_name;
                    package_path = path;
                    break;
                }
            }
        }

        if package_name.is_empty() {
            debug!(
                "Project {} has no valid Python package in src/",
                project_name
            );
            return Ok(None);
        }

        // Confirm that the package can be imported
        let is_valid = self.check_package_importable(project_dir, &package_name);
        let error = if !is_valid {
            Some(format!("Package {} cannot be imported", package_name))
        } else {
            None
        };

        Ok(Some(MCPProjectInfo {
            name: project_name,
            project_path: project_dir.to_path_buf(),
            package_name,
            package_path,
            metadata,
            is_valid,
            error,
        }))
    }

    /// Parse pyproject.toml to extract metadata
    fn parse_pyproject_toml(&self, path: &Path) -> io::Result<MCPProjectMetadata> {
        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut metadata = MCPProjectMetadata::default();

        // Use a simple approach to extract basic info
        // For production, consider using a proper TOML parser
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("name") {
                if let Some(value) = Self::extract_toml_value(line) {
                    metadata.name = Some(value);
                }
            } else if line.starts_with("version") {
                if let Some(value) = Self::extract_toml_value(line) {
                    metadata.version = Some(value);
                }
            } else if line.starts_with("description") {
                if let Some(value) = Self::extract_toml_value(line) {
                    metadata.description = Some(value);
                }
            } else if line.starts_with("mcp-capabilities") {
                // Parse array of capabilities
                if let Some(value) = Self::extract_toml_array(line) {
                    metadata.mcp_capabilities = value;
                }
            }
        }

        // Extract dependencies
        // This is a simplified approach; a real implementation would need to handle
        // more complex TOML structures properly
        if let Some(deps_section) = content.find("[dependencies]") {
            let deps_content = &content[deps_section..];
            let end_section = deps_content.find("\n[").unwrap_or(deps_content.len());
            let deps_content = &deps_content[..end_section];

            for line in deps_content.lines().skip(1) {
                // Skip the [dependencies] line
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('[') {
                    if let Some(dep) = line.split('=').next() {
                        metadata.dependencies.push(dep.trim().to_string());
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Helper to extract a simple string value from a TOML key-value pair
    fn extract_toml_value(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }

        let value = parts[1].trim();
        if value.starts_with('"') && value.ends_with('"') {
            Some(value[1..value.len() - 1].to_string())
        } else {
            Some(value.to_string())
        }
    }

    /// Helper to extract an array from a TOML key-value pair
    fn extract_toml_array(line: &str) -> Option<Vec<String>> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }

        let value = parts[1].trim();
        if value.starts_with('[') && value.ends_with(']') {
            let array_content = &value[1..value.len() - 1];
            let items: Vec<String> = array_content
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    if s.starts_with('"') && s.ends_with('"') {
                        s[1..s.len() - 1].to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect();
            Some(items)
        } else {
            None
        }
    }

    /// Check if a Python package can be imported
    fn check_package_importable(&self, project_dir: &Path, package_name: &str) -> bool {
        let result = Command::new("uv")
            .arg("pip")
            .arg("run")
            .arg("--")
            .arg("python")
            .arg("-c")
            .arg(format!("import {}", package_name))
            .current_dir(project_dir)
            .output();

        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Install a Python package's dependencies using uv
    pub fn install_dependencies(&self, project_dir: &Path) -> io::Result<bool> {
        info!(
            "Installing dependencies for project at {}",
            project_dir.display()
        );

        let result = Command::new("uv")
            .arg("pip")
            .arg("install")
            .arg("--editable")
            .arg(".")
            .current_dir(project_dir)
            .output()?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            error!("Failed to install dependencies: {}", stderr);
            return Ok(false);
        }

        info!(
            "Successfully installed dependencies for {}",
            project_dir.display()
        );
        Ok(true)
    }

    /// Create Python config data from a discovered project
    pub fn create_python_config_data(
        &self,
        project_info: &MCPProjectInfo,
    ) -> (String, PathBuf, String, Option<String>) {
        let name = project_info.name.clone();
        let project_path = project_info.project_path.clone();
        let package_name = project_info.package_name.clone();

        // Get description for environment variables if available
        let description = project_info.metadata.description.clone();

        (name, project_path, package_name, description)
    }

    /// Convert discovered projects to server configs
    ///
    /// Note: This function returns a vector of tuples containing the necessary information
    /// to create MCPServerConfig objects without directly depending on that type.
    pub fn to_server_configs(&self) -> Vec<(String, Option<String>, Vec<String>)> {
        self.projects
            .iter()
            .filter(|p| p.is_valid)
            .map(|p| {
                (
                    p.name.clone(),
                    p.metadata.description.clone(),
                    p.metadata.mcp_capabilities.clone(),
                )
            })
            .collect()
    }

    /// Get a specific project by name
    pub fn get_project(&self, name: &str) -> Option<&MCPProjectInfo> {
        self.projects.iter().find(|p| p.name == name)
    }

    /// Create a new Python MCP project from a template
    pub fn create_project(&self, name: &str) -> io::Result<PathBuf> {
        let project_dir = self.base_dir.join(name);

        // Create project directory
        fs::create_dir_all(&project_dir)?;

        // Create src directory
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create package directory
        let package_dir = src_dir.join(name);
        fs::create_dir_all(&package_dir)?;

        // Create __init__.py
        let init_py = package_dir.join("__init__.py");
        fs::write(
            &init_py,
            r#""""
MCP Server package.
"""

__version__ = "0.1.0"
"#,
        )?;

        // Create main.py
        let main_py = package_dir.join("main.py");
        fs::write(
            &main_py,
            format!(
                r#"#!/usr/bin/env python3
"""
Main entry point for the {} MCP server.
"""
import sys
import argparse
import logging

def main():
    """Run the MCP server."""
    parser = argparse.ArgumentParser(description="{} MCP server")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    log_level = logging.DEBUG if args.debug else logging.INFO
    logging.basicConfig(
        level=log_level,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    logger = logging.getLogger(__name__)
    
    logger.info("Starting {} MCP server")
    logger.info("Server started")
    
    # Your server code here
    
    return 0

if __name__ == "__main__":
    sys.exit(main())
"#,
                name, name, name
            ),
        )?;

        // Create pyproject.toml
        let pyproject_toml = project_dir.join("pyproject.toml");
        fs::write(
            &pyproject_toml,
            format!(
                r#"[build-system]
requires = ["setuptools>=42", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "{}"
version = "0.1.0"
description = "{} MCP Server"
readme = "README.md"
requires-python = ">=3.8"
license = {{ file = "LICENSE" }}
authors = [
    {{ name = "Your Name", email = "your.email@example.com" }}
]

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
                name, name
            ),
        )?;

        // Create README.md
        let readme_md = project_dir.join("README.md");
        fs::write(
            &readme_md,
            format!(
                r#"# {} MCP Server

A Model Calling Protocol (MCP) server implementation.

## Installation

```bash
uv venv
uv pip install -e .
```

## Running

```bash
python -m {}
```
"#,
                name, name
            ),
        )?;

        Ok(project_dir)
    }
}
