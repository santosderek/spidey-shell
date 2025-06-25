# Jira MCP Server Documentation

Welcome to the Jira MCP Server documentation. This package provides a Model Calling Protocol (MCP) server that integrates with Jira for ticket management and retrieval.

## Overview

Jira MCP Server is a Python package that implements the Model Calling Protocol (MCP), allowing AI models and applications to interact with Jira in a standardized way. It provides tools for retrieving, displaying, and managing Jira tickets with support for both traditional MCP interface (stdin/stdout) and a modern REST API via FastAPI.

### Key Features

- Fetch tickets assigned to the current user
- List recently updated tickets
- View detailed ticket information
- Retrieve ticket comments
- Format tickets as markdown for easy display
- REST API endpoints for all functionalities
- Standard MCP interface for compatibility with MCP clients

## Modes of Operation

The server can be run in two distinct modes:

1. **Standard MCP Mode**: Communicates via stdin/stdout following the MCP protocol, suitable for integration with CLI tools and MCP clients.

2. **Web Server Mode**: Provides REST API endpoints via FastAPI, allowing integration with web applications and services.

## Documentation Sections

### [User Guide](./user-guide/getting-started.md)

Documentation for users who want to use the Jira MCP Server:

- [Getting Started](./user-guide/getting-started.md)
- [Configuration](./user-guide/configuration.md)
- [Standard MCP Mode](./user-guide/standard-mode.md)
- [Web Server Mode](./user-guide/web-mode.md)
- [API Reference](./user-guide/api-reference.md)
- [Troubleshooting](./user-guide/troubleshooting.md)

### [Developer Documentation](./developer/architecture.md)

Documentation for developers who want to extend or modify the Jira MCP Server:

- [Architecture Overview](./developer/architecture.md)
- [Module Reference](./developer/modules.md)
- [Extending the Server](./developer/extending.md)
- [Testing Guide](./developer/testing.md)
- [Contributing Guidelines](./developer/contributing.md)
- [API Reference](./developer/api-reference.md)

## Quick Start

```bash
# Install the package
uv venv
uv pip install -e .

# Run in standard MCP mode
python -m jira_mcp

# Run in web server mode
python -m jira_mcp --mode web --port 8000
```

For detailed installation and usage instructions, see the [Getting Started](./user-guide/getting-started.md) guide.