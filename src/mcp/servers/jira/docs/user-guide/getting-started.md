# Getting Started with Jira MCP Server

This guide will help you get up and running with the Jira MCP Server.

## Prerequisites

Before installing the Jira MCP Server, ensure you have:

1. Python 3.11 or higher
2. `uv` or `pip` package manager
3. Access to a Jira instance
4. Jira API token or password

## Installation

### Using `uv` (Recommended)

```bash
# Create a virtual environment
uv venv

# Activate the virtual environment
source .venv/bin/activate  # On Unix/macOS
.venv\Scripts\activate     # On Windows

# Install the package
uv pip install -e .
```

### Using `pip`

```bash
# Create a virtual environment
python -m venv .venv

# Activate the virtual environment
source .venv/bin/activate  # On Unix/macOS
.venv\Scripts\activate     # On Windows

# Install the package
pip install -e .
```

## Configuration

The Jira MCP Server requires the following environment variables to be set:

| Variable | Description | Example |
|----------|-------------|---------|
| `JIRA_URL` | The URL of your Jira instance | `https://your-domain.atlassian.net` |
| `JIRA_API_USER` | Your Jira username (email) | `user@example.com` |
| `JIRA_API_TOKEN` | Your Jira API token | `your-api-token` |

You can set these environment variables directly in your shell:

```bash
export JIRA_URL="https://your-domain.atlassian.net"
export JIRA_API_USER="user@example.com"
export JIRA_API_TOKEN="your-api-token"
```

Or create a `.env` file in the root directory of your project:

```
JIRA_URL=https://your-domain.atlassian.net
JIRA_API_USER=user@example.com
JIRA_API_TOKEN=your-api-token
```

The server will automatically load these variables from the `.env` file if present.

## Running the Server

The Jira MCP Server can be run in two modes:

### Standard MCP Mode

This mode operates via standard input and output, following the MCP protocol:

```bash
python -m jira_mcp
```

### Web Server Mode

This mode exposes HTTP endpoints using FastAPI:

```bash
python -m jira_mcp --mode web --port 8000 --host 0.0.0.0
```

Options:
- `--mode`: Specify `web` for web server mode (default: `standard`)
- `--port`: Port to listen on (default: `8000`)
- `--host`: Host to bind to (default: `0.0.0.0`, meaning all interfaces)
- `--debug`: Enable debug logging

## Verifying Installation

### Standard Mode
In standard mode, the server will start and wait for input. You can test it by sending a JSON message:

```bash
echo '{"command": "list-commands"}' | python -m jira_mcp
```

You should receive a JSON response with the available commands.

### Web Mode
In web mode, you can verify installation by accessing the root endpoint in your browser:

```
http://localhost:8000/
```

You should see a JSON response with server information.

## Next Steps

- [Configuration](./configuration.md) - Learn more about configuring the server
- [Standard MCP Mode](./standard-mode.md) - Using the standard MCP interface
- [Web Server Mode](./web-mode.md) - Using the FastAPI web interface
- [API Reference](./api-reference.md) - Complete API reference